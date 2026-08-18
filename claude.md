# Blockwire — Master Design & Handoff Document

> Single source of truth for handing Blockwire to **Claude Code**. Covers the
> project, how to build / validate / ship it, the current state, the data model,
> known issues, and the full 4-stage plan to online.
> Companion: `Blockwire-Design-Doc.md` holds the deeper design rationale (§ refs below).

**Current version:** v0.8.33 (`blockwire-online/`) / v0.8.27 (`public/blockwire.html`) ·
**Milestone:** bwplayer (0.8) complete; pre-online cleanup done; Stage 1 Milestones 1–4
(fork + desktop shell; accounts + device linking; multi-world saves + menu redesign;
LAN multiplayer) all built; the Milestone 3 main menu was then **rebuilt from scratch**
after Riz flagged it didn't match the intended design (see Milestone 3.1 below), then
had a quick contrast fix (Milestone 3.2) — **Stage 1 is now feature-complete**, pending
Riz's visual QA and the real cross-machine LAN test on the desktop-specific pieces.
The **Camera Block** (Milestone 5, `blockwire-online`-only) is also now built, with a
same-session correction to how it's obtained (Milestone 5.1: a `/camera` chat command,
not a Workbench recipe) — see below — while that QA is pending.
**Studio:** Brain Dump Inneractive (Riz / mrrzone) · **License:** AGPLv3, source-available.
**Philosophy:** *"The player, not the payout."*

---

## 0. Working model (read first)

- **Blockwire is one self-contained HTML file** (`blockwire.html`, ~6.4k lines) using
  **three.js r128** from CDN. No build step, no bundler. Everything — world, UI,
  meshing, physics, painter, machines, player — lives in one `<script>` IIFE.
- **Riz directs and does ALL visual testing.** Claude (and Claude Code) **cannot
  render** — you validate *logic/load* only, then Riz plays it and reports.
- **Validate every change** with the Node harness before shipping (see §2). It catches
  syntax + load-time runtime errors. It does **not** catch visual/gameplay bugs.
- **Precision over cleverness.** Follow specs literally; when a spec is ambiguous,
  flag it rather than improvising. Recurring past failures were all *substituted
  design decisions* (hotkey instead of a block, wrong layout, etc.).

---

## 1. Files & deploy

**Repo / working files:**
- `blockwire.html` — the game (the one file that matters).
- `index.html` — landing page (blockwire.web.app). Canonical CSS tokens, roadmap,
  footer version, SEO tags (title/description/OG/Twitter/canonical).
- `source.html` — source viewer page.
- `robots.txt`, `sitemap.xml` — SEO (allow-all + 2 URLs). Live on Firebase.
- `assets/` — 12 machine-art JSON files (generator, chest, miners, smelter, etc.).
- **Music removed** (v0.8.17) — no `.ogg` in the deploy anymore.
- `og-image.png` — **referenced but not created**; Riz to add a 1200×630 PNG.

**Deploy:** Firebase Hosting → `https://blockwire.web.app`. Upload the folder
(index.html, blockwire.html, source.html, robots.txt, sitemap.xml, assets/) and
`firebase deploy`. Search Console: verified + indexed (URL-prefix property, HTML-file method).

---

## 2. Build / validate / ship ritual

**There is no compiler.** Validation = a Node harness that stubs THREE + the DOM,
extracts the main `<script>`, and runs it to catch load-time errors.

```bash
# Validate (SYNTAX OK / RUNTIME OK):
timeout 60 node /tmp/runcheck4.mjs 2>&1 | tail -3
```

The harness stubs: `THREE` (Proxy), `document`/`window`/`localStorage`/`Audio`/
`Image`/`fetch`/`Blob`/`FileReader`, canvas 2D context. **Quirk:** `navigator` is
read-only in Node — set it via `Object.defineProperty(globalThis,'navigator',…)`,
not `Object.assign`. It only checks *load* (rAF is stubbed; `animate()` never runs).
Rebuild it from this spec if the container resets.

**Ship steps (every version):**
1. Bump the version string in `blockwire.html`: `>vX.Y.Z<`, `Blockwire X.Y.Z Playtest`,
   `Playtest X.Y.Z ·`.
2. Bump `Playtest X.Y.Z<` in `index.html` footer, **and update the index roadmap
   section to reflect the new state** (required every ship).
3. Run the harness → must be `SYNTAX OK` / `RUNTIME OK`.
4. Copy the working file to `blockwire.html`, stage the deploy folder, zip it, hand off.

**Versioning (locked):** middle digit = major milestone, last digit = bugfix/minor.
Stages to 1.0: `0.7.x` → `0.8.0` major (bwplayer) → `0.8.x` bugfix → `0.9.0` pre-online
(LAN + client/server refactor) → `0.9.x` → `1.0.0` online → `1.0.x` → polish/publish.

---

## 3. Current state — what's built

**World / core:** procedural terrain, mining (0.5s flat break, no progress bar),
building, per-axis collision, gravity `-18`, jump `6.5`, walk `4.5` (sprint ×1.65,
crouch ×0.42), auto-step 1 block, held-key run-jump. First/third-person.

**Painter (recipe kinds):** `block` (6 painted 16×16 faces), `voxel` (3D sculpt),
`door` (oriented 1×2 swinging panel), **`plane`** (fixed 2-sided flat panel — front/back
16×16, zero width, orientation from placement face). Palette (27) + native color
picker, brush/erase/fill/eyedropper, opacity, undo/redo, transparency. Recipes are
**produced** into placeable custom blocks.

**Market / industry:** coins, chests (hold any item), generators/miners/smelters/
pipes/wire/solar, power + item networks. Machines are finite (cost iron ingots — this
is intentional; only the *text* was removed).

**bwplayer (0.8):**
- 2D billboard **layer compositor** — 15 layers × 4 directional sprites (32×64),
  composited into front/back/left/right.
- **Skin Editor** (block ☻) — paints 4 skin layers (base_skin/skin_face/facial_hair/
  hair) on a body-shape *reference*, full painter toolset, opacity, undo/redo, a
  saved-skins library, produce, save/load JSON (part or whole). Edits a **draft**,
  decoupled from the live player.
- **Equip** — inventory horizontal split: All Blocks (scroll) left; character preview
  + 2-column slot grid right (col 1: costume/hair/facial/face; col 2: clothes). Slots
  are real item-slots — produce a part → item → place in the matching slot → equipped.
  Persists (`localStorage`).
- **Clothes Maker** (block ✂) — same editor engine, 11 clothing layers; produces
  clothes that fill the equip clothes column.
- **Item Maker** (block ❖) — self-contained 16×16 painter → decorative inventory items.
- **Filler tool** (free, All Blocks) — left/right-click two same-block corners, press
  `F` to box-fill the volume (cap 12,000).

**Block types:** AIR 0, BASE_GROUND 1, COLOR_BLOCK 2, WORKBENCH 3, PAINTER 4,
CUSTOM_BLOCK 5, BEDROCK 6, ADV_WORKBENCH 7, DOOR 8, GENERATOR 9, GOLD_MINER 10,
COIN_STAMPER 11, CHEST 12, PIPE 13, WIRE 14, COAL_MINER 15, OIL_EXTRACTOR 16,
SOLAR_PANEL 17, TORCH 18 (Lamp), LAMP 19 (Bulb), IRON_MINER 20, SMELTER 21,
SKIN_EDITOR 22, CLOTHES_MAKER 23, ITEM_MAKER 24, PLANE 25.

---

## 4. Data model notes (critical for online)

- **Recipes** live in a `recipes` Map (id → recipe). Serialized *with full data* into
  the world save (`recipeToJSON`, embeds id + faces + kind + `hidden`).
- **Produce = snapshot** (v0.8.22): producing a block freezes a **hidden snapshot
  copy** of the recipe (reused per version, remade on edit). A produced/placed block
  carries its own recipe, so editing/deleting the original never touches it.
- **Soft-delete** (v0.8.21): deleting a recipe sets `hidden=true` (kept for placed
  blocks + the save), hidden from the list. Load-time safety net: a cell referencing a
  truly-missing recipe becomes a removable magenta placeholder, never an invisible ghost.
- **World save is self-contained** — embeds every referenced recipe. Important because
  online = more save/load traffic.
- **Skins/equip** persist in `localStorage`: `blockwire.skins.v1` (saved parts per
  category), `blockwire.equip.v2` (equipped item per slot).
- Doors/planes keep per-cell state in their own Maps (`doors`, `planes`), saved as
  `outDoors`/`outPlanes` and rebuilt on load.

---

## 5. Conventions & locked decisions

- **Every maker/customizer is a placeable BLOCK/station**, never a hotkey.
- **Draft/live decoupling** — editors mutate a draft; the live player/world changes
  only on explicit produce/equip.
- **New complex painters are built ISOLATED**, not by extending the block painter
  (which is tangled — 6-face grids + voxel RLE).
- **iron == silver** — canonical name is *iron*; if Riz says "silver," read "iron."
- **Combat → community-mod territory (post-1.0)**, not core. Vehicles = 1.1.
- **Moderation** (design doc §6/§10): decentralize by space (private unpoliced,
  community servers owner-moderated, official studio-enforced); recipes private so the
  shared surface is small; tools = name filter + reporting + human review + account
  accountability; one absolute line = illegal/CSAM removed + reported to NCMEC.
  Blockwire is a *tool* everywhere, a *host* only on official servers.
- **ToS** (in-game gate + TERMS.md): streaming ads/donations fine; paywalled needs
  email; **no real-money selling** of maps/worlds/recipes; custom clients fine except
  unfair advantage (esp. PvP); servers open so owners can't alter code for advantage
  or steal recipes; "what you make is your responsibility."
- **Onboarding/tutorial = 1.0 polish** (in-game + video). A tutorial built earlier
  goes stale; the machines especially need their teaching moment at 1.0.

---

## 6. Known issues & queued work (pre-online)

**Closed (v0.8.27, `public/blockwire.html`):** Riz reported recipes he'd deleted kept
coming back and newly-painted art kept disappearing, "no matter how many times I
delete it then save." Confirmed the deployed site was byte-identical to local source
(ruled out a stale-deploy explanation) and that delete/save/load logic were each
individually correct. Root cause: the public build uses a single `localStorage` save
slot with an unconditional autosave (30s interval + `beforeunload`) — if the page is
open in more than one tab (or a tab gets discarded-and-restored by the browser under
memory pressure), a stale tab still holding the *old* recipes in memory silently
overwrites a newer save the moment it autosaves or finally closes. This is exactly
what "deleted art keeps reverting no matter how many times I save" looks like from the
user's side. Fixed with a staleness guard in `saveToBrowser()`: each tab remembers the
`savedAt` timestamp of the save it last loaded or wrote (`lastKnownSavedAt`); before
writing, it peeks the *current* value in `localStorage` — if that doesn't match what
this tab last knew about, another tab has saved more recently, so this save is skipped
(with an on-screen warning telling the player to reload the tab) instead of clobbering
the newer work. Verified directly in the browser console: seeded a save, simulated a
second tab writing a newer one, then fired a stale tab's `beforeunload` autosave and
confirmed the newer save survived untouched (and the warning banner appeared); also
confirmed three consecutive normal same-tab saves still succeed with no false-positive
warnings. Harness-validated (`SYNTAX OK`/`RUNTIME OK`). Shipped as v0.8.27 and deployed.

**Closed (v0.8.22–0.8.25):** the hole-in-a-design mesh glitch (was already fixed by
the time this session checked it — `occludes()` excludes custom blocks/doors from
terrain face-culling, and `getRecipeMaterials()` already uses `DoubleSide` +
transparency for recipes with any transparent pixel); the queued behavior batch
(popping place animation, trash can, directional chests, dropped-item inventory
preview, voxel mesh-vs-cube hitbox option — all shipped); and two chest bugs found
during that work (breaking a chest only spilled resources, not full items; shift-click
quick-move into/out of a chest didn't work for full items either) — both root-caused
to code that was never updated when "chest holds full items" was added, both fixed.

**Lighting (deferred, "fine for now"):** it's an area tint, not a real lighting model
— blocks don't occlude light / cast block-shadows, and light radiates from center
rather than surrounding. A meatier rework; not before online.

**Pointer lock broken in the desktop app (blockwire-online, macOS, not our bug):**
`requestPointerLock()` doesn't work at all under Tauri's macOS WebView (WKWebView) —
confirmed the game's pointer-lock code is byte-identical to the public build's working
version, so this is a platform limitation, not a regression. Long-standing open upstream
issue: [tauri-apps/wry#445](https://github.com/tauri-apps/wry/issues/445) (open since
2022, still unresolved as of mid-2026 — WebKitGTK/Linux X11 got a partial fix, WKWebView/
macOS has "no movement" per the wry maintainers). First-person mouse-look just doesn't
lock in the desktop build until Tauri/WebKit fix this upstream. Decision: leave it as a
known limitation for now (options considered: default to third-person in the desktop
app; hack a movementX/Y-without-lock fallback — cursor stays visible/can hit window
edges, uncertain payoff). Revisit if Tauri ships a real fix, or if this becomes blocking.

---

## 7. ═══ THE 4-STAGE PLAN TO ONLINE ═══

Online is built on a **fork**, not the public single-file build. The public
`blockwire.web.app` stays the free browser game; the online client is a desktop app.

**The four stages alternate build → debug → build → debug.** The two big *build*
stages are **1** (online foundation) and **3** (public servers/ops); **2** and **4**
are their dedicated *debugging* passes.

### Stage 1 — Online foundation (accounts + app + LAN/chat/single-host)

1. **Fork Blockwire into a new folder** inside the Blockwire project directory
   (e.g. `blockwire-online/`). All online features go here — **not** the public build.
2. **Package it as a desktop app** — `.app` (macOS) / `.exe` (Windows) / Linux binary.
   (Tauri is the natural fit: wraps the HTML/JS, small binaries, cross-platform.)
3. **Add `download.html` on Firebase** — a page to download the app builds.
4. **Accounts — website sign-in + app linking.** The app requires you to **sign in on
   the website** and **link** that account to the app.
   - **Sign-up fields:** `username`, `displayName`, `email`, `password`.
   - **Privacy model:**
     - `username` + `email` are **private** (email is required for password reset).
     - `displayName` is the **only public** field.
   - **Display-name change is rate-limited: once per week.**
   - **Storage:** `email` + `password` go into **Firebase Auth** automatically.
     `displayName` + `username` are stored **separately** (e.g. Firestore) but **linked
     to the auth account** (keyed by the Firebase uid).
5. **Networking (this stage):** **LAN**, **text chat**, **hosting a server** (only for
   **single-player / "fake single-player"**), and **joining**.
   - Multi-person hosting is **NOT** in this stage — that's 1.0.

**Milestone 2 progress (accounts foundation + device linking) — built, live:**
- Firebase Auth enabled (email/password + Google) and Firestore created (`nam5`) —
  done by Riz in the console. Rules deployed from `firestore.rules` (repo root).
- Schema: `users/{uid}` = `{ username, createdAt }`, **private**, owner-read-only.
  `profiles/{uid}` = `{ displayName, lastDisplayNameChangeAt }`, **public** read, owner
  write, rules-enforced 7-day rate limit on `displayName` changes. `usernames/{lower}`
  = `{ uid }`, a uniqueness-reservation doc (Firestore has no native unique-field
  constraint) — anyone can read (client-side availability check), but it's create-once,
  and rules block a uid from claiming a second one. All three docs are written together
  in one atomic batch at signup, so a username collision fails the whole signup cleanly.
  Split into `users` (private) vs `profiles` (public) because Firestore rules can't
  restrict individual fields within a single document, only whole documents.
- `public/account.html` — new page, same visual system as `index.html`. Sign In /
  Sign Up are two separate tabs. **Email sign-up asks for email + password + username +
  display name all in one form** (validated and written atomically, username-availability
  checked *before* the Auth account is created so a taken name can't strand an
  account with no profile). **Google sign-in is different** — Google doesn't give us a
  username/display name, so a first-time Google sign-in routes to a one-time "Finish
  setting up" step asking for just those two. That same fallback step also covers the
  rare case where an email signup's profile write didn't land (auth account exists,
  Firestore write failed) — routing is driven purely by "does `users/{uid}` exist yet",
  so it self-heals regardless of which path got them there. Linked from the main nav
  (`index.html`, "Sign in").
- **Device linking** — the desktop app has no Firebase Auth session of its own, so
  linking works via a short pairing code instead: `account.html` (authenticated) mints
  `linkCodes/{code}` = `{ uid, status:'pending', createdAt, expiresAt }`; the app
  (unauthenticated) reads it and flips `status` to `'linked'` — the *only* write the
  rules let an unauthenticated client make, and only that one field. The app then reads
  the public `profiles/{uid}` for the display name and stores `{ uid, displayName }`
  locally (`localStorage` key `blockwire.account.v1`) — that local record is the whole
  of "being linked" for now, sufficient for LAN/chat identity later. A live Firebase
  Auth session inside the app is a later concern, not needed for Stage 1.
  New "Account" panel/button in the desktop app's main menu (`blockwire-online/app/`).
  **Gotcha hit and fixed:** the rules originally required `expiresAt == request.time +
  duration.value(10,'m')` (exact equality) — impossible to satisfy since the client
  can't predict the server's exact `request.time` in advance. Fixed to a range check
  (`> request.time` and `<= request.time + 11m`). Also hit a client-side miss: a new
  panel has to be added to the `PANELS` array (`showPanel()`'s allow-list) or it never
  actually shows — first attempt silently no-opped.
- Verified live end-to-end, twice (accounts, then the full link round-trip: website
  generates a code → app consumes it → local link stored → unlink works), using
  throwaway test accounts, cleaning up the Firestore docs afterward via
  `firebase firestore:delete` (the throwaway Auth users themselves were left — no CLI
  single-user-delete — harmless, no profile data attached to either).

**Milestone 1 progress (fork + desktop shell) — built, not yet Riz-QA'd:**
- `blockwire-online/` exists alongside `public/` (untouched). Structure:
  `app/` (`index.html` forked from `blockwire.html`, `assets/`, vendored
  `vendor/three.min.js` — no CDN dependency) + `src-tauri/` (Tauri v2 Rust backend,
  `tauri-plugin-fs` registered, capability scoped to `$APPDATA` only).
- World saves: `localStorage` in a plain browser tab, or the OS filesystem
  (`$APPDATA/world.json` via the fs plugin) when `window.__TAURI__` is present — same
  `serializeWorld()`/`deserializeWorld()` either way, only the transport branches.
  Window-close is intercepted (`onCloseRequested`) so the save finishes before the
  app actually quits, instead of a racy fire-and-forget on `beforeunload`.
- Dropped: the two "back to site" links and the already-dead `MUSIC_TRACKS`/
  `playRandomTrack` code (not touched in `public/` — separate, pre-existing cruft).
- **Run it:** `cd blockwire-online && npx tauri dev` (first run compiles ~350 Rust
  crates, ~1 min). Confirmed: compiles clean, launches, no console errors; the
  `app/index.html` fork also sanity-checked standalone in a plain browser (boots,
  world save/load round-trips via the localStorage path).
- **Still open:** Windows/Linux builds (macOS arm64 only so far). App icon and the
  `hidden`-attribute CSS bug are both resolved — see Milestone 3.

**Milestone 3 progress (bug batch + multi-world saves + menu redesign) — built:**
- **App icon**: real icon set generated via `npx tauri icon ~/logo.png` (512×512),
  replacing Tauri's placeholders across macOS/Windows/iOS/Android targets.
- **Fonts vendored** (`blockwire-online` only): same CDN-dependency problem three.js
  had — Archivo Black/IBM Plex Mono/IBM Plex Sans were loading from
  fonts.googleapis.com, so the logo rendered in a fallback system font whenever that
  request was slow/blocked. Downloaded the latin-subset `.woff2` files into
  `app/vendor/fonts/` with a local `fonts.css`; `index.html` now links that instead of
  the Google Fonts CDN.
- **Item Maker color picker**: was already present in the DOM (`<input type=color>`)
  but had no visible label or tooltip — identical in appearance to the adjacent
  "current color" swatch, so it read as decoration, not a control. Fixed in both
  `public/blockwire.html` and the fork by adding `title="Any color"`, matching the
  Skin Editor's already-working version of the same picker.
- **Pointer lock confirmed NOT a code bug**: `requestPointerLock()` doesn't work under
  Tauri's macOS WebView (WKWebView) at all — long-standing open upstream issue
  ([tauri-apps/wry#445](https://github.com/tauri-apps/wry/issues/445), unresolved as
  of mid-2026). Decision: leave as a known limitation for now (see §6).
- **Real, systemic CSS bug fixed**: `[hidden] { display: none !important; }` added
  globally in `blockwire-online/app/index.html`. Root cause: the `hidden` attribute's
  built-in browser rule and a plain class selector (e.g. `.mbtns { display: grid }`)
  have equal CSS specificity, so our own later-loaded class rules were silently
  winning and showing "hidden" elements anyway. This had been merely cosmetic before
  (an always-hidden Continue button showing) but became a real functional bug once
  the menu redesign needed multiple mutually-exclusive nav levels — without the fix,
  Primary nav, the Play submenu, AND the worlds list all rendered on top of each
  other simultaneously.
- **Multi-world save system** (`blockwire-online` only): replaced the single
  `world.json` slot with `worlds/{slug}.json` files + a lightweight `worlds/index.json`
  manifest (`[{slug,name,updatedAt}]`, so listing worlds never requires reading full
  multi-MB world files). Current-world pointer lives in `localStorage`
  (`blockwire.currentWorld.v1` = `{slug,name,named}`) — small and non-critical, so no
  fs round-trip on every switch. One-time migration folds any pre-existing single
  `world.json` into the new system as "My World" on first boot.
  - **"Name your world" popup**: shows once, on the first "Save and quit to menu" of
    a freshly-created world. Gotcha hit and fixed: initially gated on "is this slug in
    the index yet", but the 30s autosave interval already writes to the index for
    crash-safety long before the player quits — so that check let the prompt get
    silently skipped on any session over 30s. Fixed by adding an explicit `named`
    flag to the current-world pointer, set only when the player has actually been
    through the naming dialog (or is loading/importing/renaming a world that already
    has one) — decoupled from "has data been written to disk", which autosave is
    always free to do.
  - New fs capabilities granted: `fs:allow-read-dir`, `fs:allow-remove`,
    `fs:allow-rename` (also fixed a live bug: `clearBrowserSave()` already called
    `plugin:fs|remove` without the matching capability ever being granted).
  - "Choose a world" is no longer a raw OS file picker — it's the "My Worlds" list
    (Load/Rename/Delete per row, modeled on `renderRecipeList()`). Importing an
    external file is still the OS file picker under the hood (a plain
    `<input type=file>` already opens one for free, no Tauri dialog plugin needed),
    now as an explicit "Import a world file…" row that prompts for a name before
    writing it into the new system.
- **Menu redesign** (`blockwire-online`'s main menu only — Settings/Controls/
  Pause keep the old single-column look): right-side primary nav (Continue/Play/
  Settings/Controls/Account); Play expands to Offline (real) / Singleplayer / Public
  Servers / My Servers (the latter three rendered visibly but disabled — "Coming
  soon" — since LAN networking and public servers don't exist yet); Offline reveals
  the left-side "My Worlds" list. Background: two `<video>` elements crossfading
  through a shuffled playlist (`vendor/menu-bg/*.mp4`, no immediate repeat) — 3
  placeholder gradient loops generated with `ffmpeg` for now; drop in real footage
  later (from the in-game Camera Block — see Milestone 5 below, now built) and
  update the filename array, no other code changes needed.
- Verified via full browser-preview walkthrough (new world → save-and-quit names it
  once → Continue resumes it → My Worlds list loads/renames/deletes correctly →
  video element loads with correct src/muted/readyState, though this sandboxed
  preview tool's autoplay restrictions block confirming actual frame playback — needs
  Riz's real browser/app to confirm visually, same caveat as pointer lock) and a
  clean `npx tauri dev` compile + launch.

**Milestone 4 progress (LAN multiplayer: Open to LAN + Join + chat) — built,
Stage 1 is now feature-complete pending real cross-machine QA:**
- Real Minecraft-style "Open to LAN": a pause-menu button starts hosting your live
  world; a friend on the same network uses "Join a Game" (Play submenu, between
  Offline and the still-disabled Singleplayer/Public Servers/My Servers) to connect
  and play together in real time — movement, block changes, chat, all live.
- **Rust backend** (`blockwire-online/src-tauri/src/lan.rs`, new): a hand-rolled
  `tokio`-tungstenite WebSocket server behind `#[tauri::command]`s
  (`start_lan_server`/`stop_lan_server`/`lan_broadcast`/`lan_send_to`/`get_lan_ip`) —
  no Tauri plugin exists for raw sockets, so this follows the same raw-invoke pattern
  already used for `fs` (`TAURI.core.invoke('command_name', args)`, no `plugin:`
  prefix needed for app-defined commands). `tokio` was previously only a *transitive*
  dependency (via the `tauri` crate itself) — added directly along with
  `tokio-tungstenite`/`futures-util`. Confirmed via the generated ACL schema that
  `core:default` (already in `capabilities/default.json`) transitively includes
  `core:event:default`, so no new capability entries were needed for the
  `TAURI.event.listen(...)` calls that receive inbound socket messages.
  Inbound client messages surface in JS as Tauri events (`lan-message`,
  `lan-client-connected`, `lan-client-disconnected`); outbound goes through the
  broadcast/send-to commands. The JS **client** role needs no Tauri plugin at all —
  the webview's native `WebSocket` global connects out directly, same as any browser
  page (confirmed working: a plain, non-Tauri browser tab loading the same
  `app/index.html` successfully opened and gracefully failed a real `ws://`
  connection during testing).
- **Sync model, deliberately scoped down from a full replicated simulation:**
  `setBlock()` (the one true choke-point for all block mutations, already true before
  this milestone) got a broadcast hook — the host's calls ARE the authoritative
  change and broadcast; a client's calls are optimistic local prediction that also
  fires a `blockRequest` to the host, which re-confirms via its own broadcast
  (`netSuppressBroadcast` prevents the confirmation from looping back into another
  request). This covers plain block place/mine instantly and correctly, but
  `setBlock` alone can't capture machine/door/plane/custom-block placement (those
  need separate mesh-creation calls — `addMachine`/`placeStationMesh`/
  `rebuildDoorMesh`/etc.) — rather than replicate every placement type over the wire,
  the host periodically (every 10s) re-pushes a full `serializeWorld()` snapshot that
  every client reloads via the *existing, already-solid* `deserializeWorld()` —
  eventual consistency for everything else, built entirely from code that already
  existed and was already tested, instead of a much larger bespoke protocol.
- **Explicit scope decisions:** inventory is never synced (stays private per player —
  no clean choke point exists for it, same finding as Milestone 3's save-system
  research); machines are host-simulated only (a client's `updateMachines` tick is
  gated off entirely; a client's machine-panel interaction won't persist past the
  next 10s resync in this pass — flagged as a real, known gap, not silently
  papered over); remote players render as simple colored-box markers with a
  screen-projected name label (`Vector3.project(camera)`, no CSS2DRenderer or sprite
  addon available), not the full skin/clothes billboard avatar system; no
  reconnection handling; LAN only, no NAT traversal/internet play; no socket
  auth/encryption (trusted local network, matching the ToS's existing "custom
  clients are fine" trust model).
- Player identity reuses Milestone 2's device-linking (`getLinkedAccount()` →
  `displayName`) when linked, else a locally-generated `Player####` placeholder.
- **Verification, honestly bounded by what I can reach:** full harness validation
  (added a `WebSocket` stub to `/tmp/runcheck4.mjs`, matching the existing pattern for
  `<video>`/dynamic-`import()` — none of these are ever called at load time, only
  inside click handlers); a clean `cargo check` and `npx tauri dev` compile + launch;
  browser-preview walkthrough of every UI path *that doesn't require a second real
  peer* (Play → Join a Game form renders/validates/shows connection errors
  gracefully; pause-menu "Open to LAN" gracefully reports "only works in the desktop
  app" outside Tauri; a real `ws://` connection attempt from a plain browser tab was
  observed failing cleanly with no unhandled exception). **What I structurally could
  not verify myself:** the actual native-window click-through (I have no tool to
  interact with the live Tauri window), and — the one thing that matters most —
  a real two-machine LAN test. Needs Riz.

**Milestone 3.1 — main menu rebuild (structural correction, not a style pass) —
built:** After Milestone 4 shipped, Riz flagged (twice, in detail) that the
Milestone 3 menu didn't match the design he'd actually asked for: it rendered as a
small centered popup over the visible game world ("looks like a pause screen, not a
main menu"), and Settings/Controls opened as their own separate modal popups instead
of living inside the menu. The real, confirmed design: three horizontal bands,
**full-screen**, top-to-bottom — top strip (logo, "BLOCKWIRE", account status);
middle split into three columns **right-to-left** (rightmost = level-1 nav: Play /
Settings / Controls / Account / Friends[disabled]; middle = level-2 nav for whichever
L1 item is active, e.g. Play → Offline / My Worlds / Join a Game / Singleplayer
[disabled] / Public Servers[disabled] / Back; leftmost = the actual content pane —
worlds list, settings form, controls reference, account panel, or the join form);
bottom strip (AGPLv3 + "by playing you agree to the ToS" + a link to read it).
Also clarified: **Offline** = true solo play, no networking; **My Worlds** = pick a
world and it auto-starts hosting it on LAN (reuses Milestone 4's `startHosting()`
verbatim, just invoked from a new entry point instead of only from the pause menu).
- `#mainMenu` (`blockwire-online/app/index.html`) went from a centered `.mpanel`
  card to a true full-viewport takeover (`position:fixed; inset:0`, independent of
  `.scrim`'s flex-centering that every other panel still uses) — new CSS classes
  `.menuTopStrip`/`.menuMid`/`.menuL1`/`.menuL2`/`.menuContentPane`/
  `.menuBottomStrip` replace the old `.menuLayout`/`.menuWorldsCol`/`.menuNavCol`.
  `.menuMid` uses `flex-direction: row-reverse` so L1 sits at the true right edge.
- **Settings/Controls/Account markup is now genuinely shared, not duplicated**: each
  was split into an outer small-popup shell (`#settingsPanel`/`#controlsPanel`/
  `#accountPanel`, unchanged, still only used when reached from Pause) plus an inner
  content div (`#settingsBody`/`#controlsBody`/`#acctBody`) that a JS function
  (`selectL1()`) `appendChild`-moves into the new main menu's content pane when
  reached from there, and `restoreBodiesHome()` moves back to its home
  `#...BodyHost` element whenever `showPanel()` switches away — same IDs, same
  listeners, same `syncSettingsUI()`/`refreshAccountPanel()`, one copy of the DOM
  node, zero risk of the two homes drifting out of sync. This is the same
  DOM-reparenting trick used earlier for other shared UI, applied here for the first
  time to solve "integrated in the main menu" vs "small popup from Pause" without
  writing the settings form twice.
- L1/L2 state lives in a small set of functions: `resetMainMenuNav()` (deselect
  everything, show the default hint), `selectL1(which)` (mount the right content,
  toggle `.active` on the clicked nav button), `clearContentPane()` (shared cleanup
  step both call). `renderWorldsList(hostOnLoad)` gained a parameter — Offline calls
  it `false` (plain `loadWorldBySlug`), My Worlds calls it `true` (load, then
  `startHosting(7777)` right after, reusing Milestone 4's function and pause-menu
  LAN-status refresh as-is).
- Top-strip account status (`#menuAcctStatus`, "Link this device" / "Linked as
  <name>") reuses `getLinkedAccount()`, refreshed from the same `refreshAccountPanel()`
  call the content-pane Account view already uses — one source of truth, two
  displays. Bottom-strip ToS link reopens the *same* `#tosGate` overlay read-only
  (Agree button hidden, Cancel relabeled "Close") rather than duplicating the terms
  text anywhere — reuses the exact existing terms content.
- **Bug caught during verification, not shipped**: the ToS-reopen click handler was
  first written inside `#tosGate`'s own early `<script>` block (right after its
  markup, near the top of `<body>`) — but `#menuTosLink` lives inside `#mainMenu`,
  parsed hundreds of lines later, so `getElementById('menuTosLink')` at that point in
  the parse returned `null` and the link silently did nothing. Fixed by exposing
  `window.tosReopenReadOnly()` from that early script (the function body only looks
  up DOM elements when *called*, not when *defined*) and wiring the actual click
  listener from the main app-shell script at the bottom of the file, where
  `#menuTosLink` is guaranteed to already exist.
- **Verification:** harness (`SYNTAX OK`/`RUNTIME OK`); `cargo check` clean (no Rust
  touched this pass — purely `app/index.html`); full browser-preview walkthrough —
  ToS gate → Agree → full-screen menu confirmed; Play → L2 column confirmed
  (Offline/My Worlds/Join a Game/disabled items/Back); Offline → worlds list renders
  in the content pane with correct "My Worlds — Offline" title; Settings/Controls/
  Account each mount into the content pane with the main menu still visible around
  them (not a separate popup) — toggled "Invert vertical look" live to confirm the
  reparented control still works; bottom-strip ToS link reopens read-only and closes
  cleanly (after finding and fixing the bug above); loaded into an actual world via
  Offline → New World; forced the Pause panel open via direct DOM state (Escape
  key didn't register through the sandboxed browser tool — a known synthetic-event
  limitation noted in earlier sessions, not a product regression) and confirmed
  Pause → Settings still opens as the small unchanged popup, Back returns to Pause,
  and afterward all three body divs were confirmed back in their home `...BodyHost`
  elements. **What I could not verify myself:** real native-window rendering/click-
  through (no tool reaches the live Tauri window) and the video background's actual
  frame playback (autoplay restrictions in the sandboxed preview, same caveat as
  Milestone 3). Needs Riz's real browser/app for final visual sign-off.

**Milestone 3.2 — main menu contrast fix — built:** Riz flagged the `.menuMid`
gradient from Milestone 3.1 as too dark, crushing the video background under most of
the content pane's width. Root cause: one blanket `linear-gradient` across the whole
row hit 0.82 alpha by 55% of the way across, and the content pane (the widest column)
sat mostly in that darker range. Fix: dropped the row-wide gradient entirely and gave
each column its own, cheaper backing instead — `.menuL1`/`.menuL2` (narrow, fixed
260px, need real contrast for the nav buttons) keep a dark tint; `.menuContentPane`
now only carries a light left-to-right fade (0.4 → 0.05 alpha) so the video reads
clearly behind the worlds list / settings / etc. Verified via browser-preview
screenshots before/after at desktop width — video visibly reads through the content
pane now, nav columns still legible.

**Milestone 5 — Camera Block (`blockwire-online`-only, not the public build) —
built, needs Riz's hands-on test:** Implements the in-game recording block Riz
described for shooting b-roll to build real menu-background clips (replacing the
ffmpeg placeholder gradients from Milestone 3): place it, interact (E) to open its
panel, "Start recording" begins filming a fixed-angle landscape view facing the
direction the block was placed toward — recording keeps running in the background
while the player walks off and builds, exactly per Riz's original description
("place it, start recording, go build a castle, come back, finish recording,
download"). "Finish & download" stops and downloads the clip; "Discard" drops it.
**Scoped to `blockwire-online` only** (per Riz's choice) since its stated purpose —
generating footage for `MENU_BG_CLIPS` — only exists there; the public build is
untouched.
- **New block type**: `CAMERA_BLOCK = 26` (next after `PLANE = 25`), added
  generically everywhere a machine type needs to plug in: `isMachine`,
  `MACHINE_COLOR`/`MACHINE_LABEL` (fallback flat-color look + HUD label),
  `machineName`, and `STATION_ASSETS` (registers `assets/cameraBlock.json` for its
  baked look).
- **Acquisition — a chat command, deliberately NOT a Workbench recipe.** First
  pass wrongly added it to `CRAFTABLES` as a normal free craft; Riz caught this
  immediately — he'd specifically said "a block accessible by a command." Fixed
  same-session (see the correction note right after this milestone): typing
  `/camera` in the chat box gives one to your inventory (`addItemToInventory`,
  first empty hotbar/backpack slot), via a new small `CHAT_COMMANDS` table in
  `sendChatMessage()` (`{name: fn(args)}` — easy to extend with more commands
  later). Unknown `/whatever` replies "Unknown command" instead of silently
  no-opping.
- **Orientation**: reuses the exact chest pattern — at placement, camera yaw is
  snapped to the nearest 90° and stored on the machine instance as `m.yaw`
  (`if (item.machine === CHEST || item.machine === CAMERA_BLOCK)` in the placement
  handler). Confirmed (by reading `serializeWorld`/`deserializeWorld`) that `m.yaw`
  was *already* generically whitelisted for save/load and that mesh orientation is
  *separately* and *already* persisted for every station mesh via `outStations`/
  `data.stations` (`mesh.rotation.y`) — no serialization code needed changing at all,
  the existing generic machinery just needed the new type routed through it.
- **Rendering**: a genuinely new technique for this codebase — the only prior
  precedent for a second live-rendered view was the voxel/skin editor's own
  isolated preview scene (`voxelEditorInit`); Camera Block instead renders the
  *actual live world* `scene` through a second `THREE.PerspectiveCamera` positioned
  at the block (offset slightly along its facing direction so it doesn't clip into
  its own mesh) into an off-DOM `<canvas>` + its own `THREE.WebGLRenderer` — same
  forward-vector convention as the player camera (`fx = -Math.sin(yaw), fz =
  -Math.cos(yaw)`) so "the direction it was placed" matches what actually gets
  filmed. That render call is hooked into `animate()` right after the main
  `renderer.render(scene, camera)` call, gated on a small `activeCameraRecordings`
  Set — **deliberately not gated on `openMachine`/menu state**, since the whole
  point is it keeps recording while the panel is closed and the player is elsewhere.
- **Capture**: `canvas.captureStream(30)` → `MediaRecorder` (`vp9`, falling back to
  the browser's default codec if unavailable) → downloads a `.webm` on finish. UI
  copy deliberately never promises "mp4" — `MediaRecorder` output format is
  browser-dependent and never guaranteed to be mp4; re-encoding/editing was already
  Riz's stated plan externally, so this doesn't block the workflow.
- **Recording state is deliberately runtime-only**: lives on the machine instance
  (`m.recording`/`m.camCanvas`/`m.camRenderer`/`m.camCam`/`m.camRecorder`/
  `m.camChunks`/`m.camStartedAt`), never touched by `serializeWorld()` (which
  already whitelists exactly which `m` fields it writes — these were simply never
  added to that whitelist, so they're excluded for free, same mechanism that already
  keeps `powered`/`wattage`/`timer` etc. out of saves). This means quitting or
  reloading mid-recording loses the in-progress clip — acceptable for what's
  effectively Riz's own one-session content-creation tool, not core gameplay state,
  but a real, known limitation worth knowing about before a long recording session.
  Breaking the block mid-recording (`removeMachine`) discards the recording rather
  than leaving an orphaned `MediaRecorder`/renderer with nothing left to stop it.
- **Panel UI**: `renderMachinePanel()` gained an early-return branch for
  `CAMERA_BLOCK` (its own `renderCameraBlockPanel()`/`wireCameraBlockPanel()`
  functions) rather than threading a new case through the miner/processor/
  power-source machinery that doesn't apply to it — idle state shows "Start
  recording"; recording state shows a live elapsed-time counter (updated every
  frame via the existing `refreshMachineLive()` hook, same place the sweeping
  minigame bar already redraws per-frame) plus "Finish & download"/"Discard".
  `machineSig()` gained a `CAMERA_BLOCK` case (`'rec'`/`'idle'`) so the panel
  re-renders on state changes exactly like every other machine type already does.
- **Placeholder art**: `assets/cameraBlock.json` generated via a small one-off
  Python script (same spirit as the Milestone 3 ffmpeg gradient placeholders) —
  a dark camera-casing gray on 5 faces, a lens ring + glass + highlight + a small
  red "recording" dot on the front face, using the exact same packed-pixel format
  as the other 12 machine-art JSONs (confirmed by reading `assets/generator.json`
  first). Swappable later without any code changes, same as the video clips.
- **Verification**: harness validation (added `MediaRecorder`/
  `canvas.captureStream()` stubs to `/tmp/runcheck4.mjs`, matching the existing
  `WebSocket`/`<video>` stub pattern — none of these run at load time, only inside
  click handlers) — `SYNTAX OK`/`RUNTIME OK`; confirmed via browser-preview network
  log that `assets/cameraBlock.json` fetches `200 OK` and `registerStationRecipe()`
  bakes it with no console errors (proves the placeholder art's pixel data is
  well-formed). **What I could not verify myself:** actually placing/interacting
  with the block in a live browser session — this sandboxed browser tool's
  synthetic OS-level key events don't register with this game's key listeners
  (`Tab`/`Escape` didn't work either, already flagged during Milestone 3.1 QA
  above) — but see Milestone 5.1 immediately below for how the `/camera` command
  path specifically *was* end-to-end verified despite that limitation. Still
  outstanding: placing the block, interacting with it in-world, and the actual
  record → walk away → finish → download loop. Needs Riz's hands, same as the LAN
  cross-machine test.

**Milestone 5.1 — Camera Block acquisition correction — built:** Riz caught that
the first pass put Camera Block in the Workbench craft menu (`CRAFTABLES`) — he'd
specifically said "a block accessible by a command," not a recipe. Fixed same
session:
- Removed the `CRAFTABLES` entry and its `.craftBtn` row from the Workbench HTML.
- Added a small `CHAT_COMMANDS` table (`{name: fn(argsArray)}`) and a
  `runChatCommand()` dispatcher, intercepted at the top of `sendChatMessage()`
  before any of the existing LAN-chat logic runs — `/camera` calls
  `addItemToInventory({type:'machine', machine: CAMERA_BLOCK})` and reports success
  or "Inventory full" via `addChatMessage(null, ...)` (the existing system-message
  styling, no `displayName` → italic).
- **Real gate found and fixed along the way**: the chat box was built LAN-only —
  `openChatInput()` returned early unless `netRole` was set, and the `Enter` keydown
  handler required `netRole` too, because chat messages only ever go anywhere over
  a LAN connection. But Riz's actual use case for `/camera` (solo b-roll shooting)
  is exactly the case where you're *not* networked. Fixed by decoupling "can open
  the box / can run a command" from "can send a chat message": `openChatInput()`
  and the `Enter` binding no longer check `netRole` at all; `sendChatMessage()`
  branches on a leading `/` *before* touching any of the `netRole === 'host'`/
  `'client'` networking code, so commands always work and plain chat text still
  correctly no-ops with a "Not connected" hint when solo.
- **Verified end-to-end**, working around the same synthetic-keydown tool
  limitation noted above by dispatching real `KeyboardEvent`s via
  `document.dispatchEvent(...)`/`input.dispatchEvent(...)` in the browser preview's
  JS console instead of the computer-input tool: confirmed `Enter` now opens
  `#chatInput` with no LAN session active; typed `/camera` → confirmed via
  screenshot the block appeared in the hotbar and the log read "Camera Block added
  to your inventory."; confirmed `/nope` replies "Unknown command: /nope"; confirmed
  plain non-command text while offline replies "Not connected — chat needs Open to
  LAN or Join a Game. (Commands like /camera still work solo.)" instead of silently
  doing nothing. Harness re-validated clean after each edit.

**Milestone 5.2 — three Riz-reported fixes (keyboard mine/place, Camera Block
recording direction, world-naming prompt) — built:**
- **Keyboard mine/place (`.` / `/`)**: Riz asked for arrow-key look (already
  existed) plus `.` to click/mine and `/` to right-click/place — a fully
  keyboard-driven control scheme, notably useful on the desktop app's macOS
  build where pointer lock doesn't work at all (§6) and mouse-look is
  unavailable. The real mine/place logic lived inline inside the `mousedown`
  handler, branching on `e.button`, so it was extracted into two standalone
  functions — `performMineClick(fromKeyboard)`/`performPlaceClick(fromKeyboard)`
  — called both by the real `mousedown` handler (unchanged behavior) and by new
  `Period`/`Slash` cases in the main `keydown` handler. The `fromKeyboard` flag
  skips one specific branch: a real mouse click in first-person without pointer
  lock just re-engages the lock instead of mining/placing — that gate makes no
  sense for a keyboard press (a keyboard-only player may never have pointer
  lock at all), so keyboard-triggered calls bypass it and act immediately.
  `.` mirrors mousedown/mouseup (hold to keep mining, via `leftHeld` + a new
  `keyup` listener); `/` is one-shot like a real right-click, guarded on
  `e.repeat` so OS key-repeat doesn't spam placements. Also updated the in-game
  hint text and the Controls reference panel to document both keys.
- **Camera Block recorded backwards — real bug, not a design choice.** Riz
  placed one and found the visible lens (baked onto the mesh's front face) was
  on the front, but the actual footage filmed *behind* it. Root cause,
  confirmed by deriving the math rather than guessing: `mesh.rotation.y = m.yaw`
  rotates the mesh's local +z ("front" texture) to world direction
  `(sin(yaw), cos(yaw))` — but `startCameraRecording()` had pointed its capture
  camera at `(-sin(yaw), -cos(yaw))`, i.e. the *player's own forward vector*
  (the same convention a chest uses so its front/opening faces back toward
  whoever placed it — correct for a chest, wrong for a camera). Fixed by
  flipping the sign so the recording camera looks the same direction the baked
  lens texture actually faces.
- **"New World" never prompting for a name — real bug, root-caused.**
  `loadWorldBySlug(slug, name)` unconditionally wrote `named: true` to the
  current-world pointer on every load, on the theory that "an existing world
  already has a real name." That's false the moment a freshly-created,
  never-yet-named world gets loaded again before its first "name your world"
  prompt — e.g. clicking Continue, or re-selecting it from the My Worlds list —
  which permanently kills the one-time prompt for that world. Worse, `named`
  was previously tracked *only* on the current-world pointer, never in the
  worlds index itself, so there was no way to even ask "was this specific
  world ever actually named" once you'd switched away from it. Fixed properly:
  `upsertWorldIndexEntry(slug, name, named)` gained a third, optional `named`
  parameter — omitted, it preserves whatever was already in the index (rename,
  periodic autosave); passed explicitly, it sets the definitive value (world
  migration and file import both pass `true`; `saveCurrentWorld()` now passes
  `cur.named` through instead of dropping it). `loadWorldBySlug` now looks up
  the real per-world `named` value from the index instead of hardcoding `true`.
- **Verified all three in the browser preview**, working around the sandboxed
  tool's synthetic-keyboard limitation (noted repeatedly above) by driving the
  UI through direct DOM `.click()` calls and dispatched `KeyboardEvent`s from
  the JS console instead: created a new world, confirmed
  `blockwire.currentWorld.v1` showed `named:false`; opened Pause (dispatched
  `Escape`) → "Save and quit to menu" → confirmed `#nameWorldOverlay` opened;
  closed it *without* naming (simulating "never named it") and confirmed the
  worlds index still correctly held `named:false` for that world (this is the
  exact state the old code silently corrupted); reloaded the page fresh,
  clicked **Continue** (the precise regression path — this calls
  `loadWorldBySlug`) and confirmed `named` stayed `false` afterward instead of
  flipping true; reopened Pause → quit again and confirmed the naming prompt
  *did* fire this time; completed it with a real name and confirmed both the
  current-world pointer and the worlds index ended up in sync (`named:true`,
  matching name). Did **not** get to interactively verify the `.`/`/` keyboard
  controls or the Camera Block direction fix inside a live render (both need a
  real WebGL frame and player position/targeting state that's impractical to
  fully fake from the console) — the direction fix is confirmed correct by
  the coordinate-geometry derivation above, but a real placement/recording
  check needs Riz's hands. Harness re-validated clean (`SYNTAX OK`/
  `RUNTIME OK`) after every edit.

**Milestone 5.3 — LAN "still connected after Stop hosting" bug (Rust) + trash
can unreachable while scrolled (CSS) — built:**
- **Remote players are still simple colored-box markers, not full avatars —
  this is the documented Milestone 4 scope decision (§ above), not a bug.**
  Riz noticed and flagged it; confirmed it's exactly the known, deliberate gap
  ("remote players render as simple colored-box markers... not the full
  skin/clothes billboard avatar system"). Upgrading to real billboard avatars
  over the network is real, separate scope (needs syncing skin/equip state
  per remote player, not just position) — not touched this pass; flag if/when
  Riz wants it prioritized.
- **Real bug, found in the Rust LAN server, not JS**: after "Stop hosting,"
  Riz's client stayed connected to the (supposedly dead) host indefinitely.
  Root cause in `blockwire-online/src-tauri/src/lan.rs`: `stop_lan_server`
  only cleared the `clients` map (the outbound-message channels) — that ends
  each connection's `write_task`, but the *separate* per-connection task
  running `read.next().await` (the read half of the split WebSocket, inside
  `handle_connection`) was never touched. With nothing telling it to stop, it
  just sat blocked forever, so the TCP connection to that client was never
  actually closed — no close frame, no dropped socket, so the client's
  `ws.onclose` never fired and it kept believing it was connected. Fixed by
  tracking each connection's `JoinHandle` in a new `tasks: TaskMap` (keyed by
  client id, populated right when a connection is accepted — moved id
  assignment out of `handle_connection` and into the accept loop so the
  handle can be filed under its id immediately) and having `stop_lan_server`
  abort every one of those tasks outright, not just clear the outbound
  channel — aborting drops the task's half of the split stream, which is what
  actually tears down the socket and lets the OS tell the client it's gone.
  Also removes a connection's own entry from `tasks` on natural disconnect
  (client leaves on their own) so the map doesn't grow across a long hosting
  session. Verified with `cargo check` (clean compile) — did **not** get a
  real two-machine "stop hosting, confirm the other side actually drops"
  test, same bounded-verification caveat as the original LAN build; needs
  Riz's hands for that specific repro.
- **Trash can unreachable once you scroll — real CSS bug.** `#trashSlot` was
  `position: absolute` inside `.invPanel`— but `.invPanel` itself scrolls
  (`overflow-y: auto; max-height: 92vh`, e.g. on a shorter window) and the
  "All Blocks" grid inside it has its own inner scroll too. An
  absolutely-positioned child scrolls along with whichever ancestor's content
  box is scrolling, so once you scrolled far enough to reach an item lower in
  the list, the trash icon (pinned near the *top* of the panel) scrolled out
  of view right when you needed it — exactly Riz's complaint. Fixed by
  switching `.trashSlot` to `position: fixed` (viewport-pinned, top-right
  corner) with a `z-index` above `#inventory`'s own overlay — confirmed no
  ancestor has a `transform`/`filter` that would've captured the fixed
  positioning instead of the real viewport. `pointOverTrash()`'s drag-drop hit
  test already used `getBoundingClientRect()` (viewport coordinates), so no
  JS changes were needed — only the CSS positioning scheme. Verified directly
  in the browser preview: opened the Tab inventory, forced both the All
  Blocks grid and the outer panel to scroll via the console, and confirmed
  the trash icon's `getBoundingClientRect()` was byte-identical before and
  after (`position: fixed` confirmed via `getComputedStyle`) — it no longer
  moves no matter what's scrolled.

**Milestone 5.4 — ported the stale-save-clobber guard into `blockwire-online`
— built:** After fixing the public build's "deleted recipes keep coming back"
bug (last-write-wins autosave + a stale second tab, see §6), Riz asked for the
same protection in `blockwire-online` for consistency. The desktop app uses a
multi-world file store rather than a single `localStorage` slot, but
`saveCurrentWorld()` had the exact same shape of gap: a 30s autosave interval,
the Tauri window-close handler, and `beforeunload` all call it unconditionally,
with nothing to stop a second instance (two app windows, or the app plus a
browser-preview tab) open on the same world file from silently overwriting the
other's newer save. Ported the identical fix: a module-level `lastKnownSavedAt`
tracks the `savedAt` this instance last loaded or wrote for whichever world is
current; `saveCurrentWorld()` peeks the on-disk/localStorage copy before
writing and skips (with the same on-screen warning) if it's moved on without
this instance knowing. Reset to `null` on `assignDefaultWorldName()` (fresh
world, nothing to conflict with yet) and set from the loaded save's `savedAt`
in both `loadWorldBySlug()` and the world-file import handler, mirroring the
public build exactly. Verified the same way: harness clean, `cargo check`
clean (no Rust touched — the guard lives entirely in the JS save path, which
already abstracts over the Tauri-fs vs. localStorage transport), and a
browser-preview repro identical to the public build's — seeded a world save,
overwrote it with a marked "newer" save, fired the stale instance's
`beforeunload` autosave, and confirmed the newer save survived untouched with
the warning banner showing.

### Stage 2 — Debugging
Stabilize everything from Stage 1: account/link edge cases, LAN reliability, chat,
save/load over the wire, the app packaging on all three OSes.

### Stage 3 — Public servers & operations
**Public servers, hosting, whitelisting, admin + mods**, and the operational tooling
that goes with running community/official spaces (per the moderation model in §5).

### Stage 4 — Debugging
Same as Stage 2: stabilize everything from Stage 3 — public servers, hosting,
whitelisting, admin/mods, and the operational tooling — before it's considered done.

*(Out of scope for these four stages, tracked separately: the onboarding/video
tutorial is 1.0 polish; combat is post-1.0 community-mod territory; vehicles are 1.1.)*

**Design notes captured for later (not built yet — recorded so they're not lost):**
- **1.0 — accounts:** password reset by email + username (the pair, since username is
  private and not itself a login credential); resetting a password signs out every
  other linked device on that account. A second identifier alongside `username`:
  a long **private ID string** you hand a friend directly (out-of-band) to send a
  friend request — distinct from `username` (also private) and `displayName` (public);
  the point is friends can add each other without a public, guessable/spammable handle.
  Needs a **Friends menu** and a **My Account/profile menu** (change display name —
  still the existing 1-week rate limit — plus whatever else profile settings end up
  living there).
- **1.0/Stage 3 — world hosting model:** a saved world gets a visibility setting,
  **Whitelist** or **Public**. Public adds a name/capacity/description and lists it in
  the public server directory. Whitelist has its own **"Friends allowed"** toggle —
  controls both whether friends can see what world you're in AND whether they can
  join it. Whitelisting a specific person happens via their **private ID, not
  displayName** (a famous/popular player's public display name would get flooded with
  join attempts otherwise) — either a chat command or a settings-panel entry.
  **Explicitly flagged as a real gap**: hosting from the "My Worlds" list (i.e. what
  Milestone 4's "Open to LAN" is) dies the moment you close the game — Public/directory
  listing implies a real always-on dedicated-hosting story, not just LAN, which is
  genuinely Stage 3 scope (§7 already says as much), not an extension of Milestone 4.
- **Stage 2 — main-menu redesign concept** (supersedes Milestone 3's two-column
  layout when we get there, not built yet): three horizontal bands top-to-bottom —
  a thin top strip (logo, "BLOCKWIRE", sign in/log in), the main panel in the middle,
  and a thin bottom strip (AGPLv3 + "by playing you agree to the ToS" + a link to open
  it). The middle panel itself splits into three columns right-to-left: **rightmost**
  = level-1 nav (Play, Settings, Friends, My Account, ...), **middle** = level-2 nav
  for whichever level-1 item is active (e.g. Play → Offline / My Worlds / Public
  Servers / My Servers / Back), **leftmost** = the actual content/action pane for
  whatever's selected (a worlds list, a settings form, etc.).
- **Downloads:** distribute Windows/Linux/Intel-Mac builds via **GitHub Releases**
  rather than hosting the binaries ourselves — free, versioned, no bandwidth ceiling
  to worry about (this is what made Firebase Hosting or Internet Archive feel like a
  stretch earlier). `download.html` (still not built) would just link to
  `github.com/<org>/<repo>/releases/latest`. Prerequisite: an actual public GitHub
  repo for the project doesn't exist yet — `public/index.html`'s "View the source"
  link is still a `<!-- TODO: point this at the real repository -->` placeholder.

---

## 8. Handoff checklist for Claude Code

- [ ] Read `blockwire.html` incrementally (it's large); read `Blockwire-Design-Doc.md`
      for rationale.
- [ ] Rebuild the Node harness (§2) if missing; validate before every change.
- [ ] Respect: one file, three.js r128, no build step, Riz does visual QA.
- [ ] Follow specs literally; flag ambiguity; makers are blocks; drafts decouple.
- [x] Before online: close the hole-in-design mesh bug + the small behavior batch.
- [ ] Stage 1 on a **fork** (`blockwire-online/`), leaving the public build intact —
      Milestone 1 (fork + desktop shell) built; accounts and LAN networking still open.
