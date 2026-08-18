// LAN multiplayer ("Open to LAN"): a plain WebSocket server embedded in the
// Tauri backend. The frontend has no Tauri plugin for raw sockets, so this is
// a hand-rolled tokio-tungstenite server behind a few #[tauri::command]s,
// mirroring the raw-invoke pattern already used for the fs plugin — see
// app/index.html's TAURI.core.invoke('plugin:fs|...', ...) calls.
//
// Inbound client messages surface in JS via a Tauri event ("lan-message");
// outbound messages go through lan_broadcast/lan_send_to. The host's own
// gameplay never goes through this module at all — only remote clients do.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

type ClientMap = Arc<Mutex<HashMap<u32, mpsc::UnboundedSender<Message>>>>;
type TaskMap = Arc<Mutex<HashMap<u32, tokio::task::JoinHandle<()>>>>;

struct LanServerHandle {
    clients: ClientMap,
    // Per-connection task handles, keyed by client id — see stop_lan_server
    // for why these need to be aborted explicitly, not just `clients` cleared.
    tasks: TaskMap,
    listener_task: tokio::task::JoinHandle<()>,
}

#[derive(Default)]
pub struct LanServerState {
    inner: Mutex<Option<LanServerHandle>>,
}

#[derive(Serialize, Clone)]
struct InboundEvent {
    #[serde(rename = "clientId")]
    client_id: u32,
    text: String,
}

#[derive(Serialize, Clone)]
struct PresenceEvent {
    #[serde(rename = "clientId")]
    client_id: u32,
}

#[tauri::command]
pub async fn start_lan_server(
    app: AppHandle,
    state: State<'_, LanServerState>,
    port: u16,
) -> Result<String, String> {
    let mut guard = state.inner.lock().await;
    if guard.is_some() {
        return Err("Already hosting.".into());
    }

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| e.to_string())?;
    let bound_port = listener.local_addr().map_err(|e| e.to_string())?.port();

    let clients: ClientMap = Arc::new(Mutex::new(HashMap::new()));
    let tasks: TaskMap = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(Mutex::new(0u32));

    let clients_for_task = clients.clone();
    let tasks_for_task = tasks.clone();
    let next_id_for_task = next_id.clone();
    let app_for_task = app.clone();
    let listener_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let clients2 = clients_for_task.clone();
                    let tasks2 = tasks_for_task.clone();
                    let app2 = app_for_task.clone();
                    // Assign the id here, before spawning, so the connection's
                    // own task handle can be filed under it immediately — that
                    // handle is what lets stop_lan_server actually kill this
                    // connection later (see stop_lan_server).
                    let id = {
                        let mut n = next_id_for_task.lock().await;
                        let v = *n;
                        *n += 1;
                        v
                    };
                    let tasks3 = tasks2.clone();
                    let conn_task = tokio::spawn(async move {
                        handle_connection(stream, addr, clients2, tasks2, id, app2).await;
                    });
                    tasks3.lock().await.insert(id, conn_task);
                }
                Err(_) => break, // listener dropped/closed — stop_lan_server tore it down
            }
        }
    });

    *guard = Some(LanServerHandle {
        clients,
        tasks,
        listener_task,
    });

    Ok(bound_port.to_string())
}

async fn handle_connection(
    stream: TcpStream,
    _addr: SocketAddr,
    clients: ClientMap,
    tasks: TaskMap,
    id: u32,
    app: AppHandle,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(s) => s,
        Err(_) => return,
    };
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    clients.lock().await.insert(id, tx);
    let _ = app.emit("lan-client-connected", PresenceEvent { client_id: id });

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(next) = read.next().await {
        match next {
            Ok(msg) if msg.is_text() => {
                let text = msg.into_text().unwrap_or_default().to_string();
                let _ = app.emit(
                    "lan-message",
                    InboundEvent {
                        client_id: id,
                        text,
                    },
                );
            }
            Ok(msg) if msg.is_close() => break,
            Err(_) => break,
            _ => {}
        }
    }

    clients.lock().await.remove(&id);
    write_task.abort();
    // Remove our own entry now that we're exiting on our own (client
    // disconnected normally) — stop_lan_server drains+aborts whatever's left
    // for the abrupt "host stopped hosting" case, so this just keeps the map
    // from growing unboundedly over a long session.
    tasks.lock().await.remove(&id);
    let _ = app.emit("lan-client-disconnected", PresenceEvent { client_id: id });
}

#[tauri::command]
pub async fn stop_lan_server(state: State<'_, LanServerState>) -> Result<(), String> {
    let mut guard = state.inner.lock().await;
    if let Some(handle) = guard.take() {
        handle.listener_task.abort();
        // Abort every per-client connection task outright. Previously this
        // only cleared `clients` (the outbound-message channels), which ends
        // each connection's write_task but leaves its read loop
        // (`read.next().await` in handle_connection) blocked forever with
        // nothing telling it to stop — the client's socket was never
        // actually closed, so it never saw a close event and kept believing
        // it was still connected long after "Stop hosting" was clicked.
        // Aborting the task drops its half of the split WebSocket stream,
        // which is what actually tears down the TCP connection.
        for (_, task) in handle.tasks.lock().await.drain() {
            task.abort();
        }
        handle.clients.lock().await.clear();
    }
    Ok(())
}

// Send to every connected client. `except` optionally skips one client id —
// used to avoid echoing a message back to the client that just sent it,
// though most of our protocol WANTS the echo (single source of truth), so
// this is usually called with `None`.
#[tauri::command]
pub async fn lan_broadcast(
    state: State<'_, LanServerState>,
    text: String,
    except: Option<u32>,
) -> Result<(), String> {
    let guard = state.inner.lock().await;
    let handle = guard.as_ref().ok_or("Not hosting.")?;
    let clients = handle.clients.lock().await;
    for (id, tx) in clients.iter() {
        if Some(*id) == except {
            continue;
        }
        let _ = tx.send(Message::Text(text.clone().into()));
    }
    Ok(())
}

#[tauri::command]
pub async fn lan_send_to(
    state: State<'_, LanServerState>,
    client_id: u32,
    text: String,
) -> Result<(), String> {
    let guard = state.inner.lock().await;
    let handle = guard.as_ref().ok_or("Not hosting.")?;
    let clients = handle.clients.lock().await;
    if let Some(tx) = clients.get(&client_id) {
        let _ = tx.send(Message::Text(text.into()));
    }
    Ok(())
}

// Best-effort LAN-reachable IPv4 address to show the host (skips loopback).
// No extra crate needed: connecting a UDP socket (without actually sending
// anything) makes the OS pick the outbound interface for us.
#[tauri::command]
pub fn get_lan_ip() -> Result<String, String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket
        .connect("8.8.8.8:80")
        .map_err(|e| e.to_string())?;
    let addr = socket.local_addr().map_err(|e| e.to_string())?;
    Ok(addr.ip().to_string())
}
