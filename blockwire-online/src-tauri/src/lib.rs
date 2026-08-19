mod lan;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .manage(lan::LanServerState::default())
    .invoke_handler(tauri::generate_handler![
      lan::start_lan_server,
      lan::stop_lan_server,
      lan::lan_broadcast,
      lan::lan_send_to,
      lan::lan_kick_client,
      lan::get_lan_ip,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
