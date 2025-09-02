# RolyPoly GUI — Operational Specification (v0.3 draft)

---

## 1. Purpose

Provide a modern, dark-themed, WinZip-style client for creating and extracting compressed archives. Must run natively on **Windows, macOS, and Linux** with identical behavior.

---

#### 2. High-Level Requirements

| #   | Requirement                                                                       | Notes                                                                |
| --- | --------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| R1  | Open existing archives (`.zip`, `.tar.gz`, `.7z`, …) and show contents.           | Plug-in architecture for future formats.                             |
| R2  | Create new archives from user-selected files/folders.                             | Support drag-and-drop and “Add” button.                              |
| R3  | Extract whole archive or selected items to a target folder.                       | Default = current working dir.                                       |
| R4  | Show file table (Name, Size, Type, Modified) with sorting & multi-select.         | Column resize / double-click auto-fit.                               |
| R5  | Display archive metadata panel (size donut, compression %, encryption algorithm). | Real-time update while building archive.                             |
| R6  | Toggle **Compress** / **Extract** CTA based on context.                           | If archive is loaded → *Extract*; if file list pending → *Compress*. |
| R7  | AES-256 encryption option when creating archives.                                 | Prompt for password with strength meter.                             |
| R8  | Persist last 10 archive locations & settings per platform (roaming-safe).         | JSON in user-config dir.                                             |
| R9  | Fully keyboard-navigable & screen-reader friendly.                                | WCAG 2.2 AA.                                                         |
| R10 | Theming engine (dark default, light optional).                                    | User toggle + auto match OS.                                         |

---

#### 3. UI Anatomy (refer to mock-ups)

| Ref | Element                           | Function                                                                    |
| --- | --------------------------------- | --------------------------------------------------------------------------- |
| A   | **Title-bar** (“RolyPoly”)        | Left-click = About dialog.                                                  |
| B1  | **Add (+ icon)**                  | Opens OS file picker, appends to file list.                                 |
| B2  | **Open (folder-arrow icon)**      | Opens archive, replaces view; Ctrl+O shortcut.                              |
| B3  | **Copy Path (clipboard icon)**    | Copies archive or selected file paths to clipboard.                         |
| B4  | **Share (paper-plane icon)**      | OS share sheet (macOS Services / Windows Share).                            |
| B5  | **Settings (gear icon)**          | Global prefs: theme, default compression level, language.                   |
| B6  | **Tools (gear-with-wrench icon)** | Power-user tools (test archive, repair, split, checksum).                   |
| C   | **File Table**                    | Multi-select, context menu (preview, remove, rename).                       |
| D   | **Archive Info Panel**            | Donut chart, comp. %, encryption algorithm.                                 |
| E   | **Output Path Bar**               | Autocomplete, drag-out for OS copy.                                         |
| F   | **Primary Button**                | Blue; label toggles `Compress` / `Extract`. Disabled until action is valid. |

Error, progress, or password dialogs surface as modal sheets with blur background.

---

#### 4. Core User Flows

1. **Open & Extract**
   `User hits Open → selects my.zip → contents populate → clicks Extract → chooses location → progress bar → success toast.`

2. **Create & Encrypt**
   `Drag files onto window → CTA swaps to Compress → clicks Compress → dialog asks optional password & level (store/fast/normal/max) → progress → archive saved → metadata panel updates.`

3. **Quick Add**
   `Archive already open → Add files → program auto-prompts to update archive (re-write).`

4. **Settings Change**
   `User opens Settings → toggles light theme → UI re-paints instantly; persisted.`

---

#### 5. States & Transitions

| State         | CTA Label           | Enabled Icons           | Info Panel Content                |
| ------------- | ------------------- | ----------------------- | --------------------------------- |
| Empty         | Compress            | Add, Open               | “Drop files or use +” placeholder |
| Building      | Compress (disabled) | none                    | Live progress, cancel btn         |
| Ready Archive | Extract             | Add, Copy, Share, Tools | Actual metadata                   |
| Extracting    | Extract (disabled)  | none                    | Live speed, ETA                   |

---

#### 6. Non-Functional

* **Performance**: ≤ 50 ms UI response; can process 4 GB archive without UI jitter.
* **Memory**: Stream compression/extraction to keep < 200 MB RAM for 4 GB archive.
* **Security**: Zero plaintext passwords on disk; use OS-keychain if “remember”.
* **Packaging**: Code-sign & notarize on macOS, MSI on Windows, AppImage/Flatpak on Linux.

---

#### 7. Recommended Rust-Centric Tech Stack

| Layer                    | Option                                                          | Why                                                                                          |
| ------------------------ | --------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| **UI Toolkit**           | **Slint**                                                       | Modern, declarative, native GPU, bindings in pure Rust, out-of-box dark mode, binary \~2 MB. |
| (Alt.)                   | Iced                                                            | Pure Rust, Elm-style, good theming; slower startup on older GPUs.                            |
| **Windowing**            | Winit                                                           | Cross-platform window/event loop, used by Slint/Iced.                                        |
| **Rendering**            | Skia-Safe *or* wgpu (via Slint default)                         | Vector & text quality; GPU accelerated.                                                      |
| **Compression back-end** | `zip-rs`, `zstd`, `7z-rust`                                     | Async where available; trait-based adapter.                                                  |
| **Crypto**               | `aes-gcm` + `argon2`                                            | Authenticated encryption; password KDF.                                                      |
| **Packaging**            | `cargo-bundle` or Tauri-Build                                   | Produce installers for all OSes.                                                             |
| **CI**                   | GitHub Actions matrix (windows-latest, macos-13, ubuntu-22.04). |                                                                                              |

> **Why not Tauri/Electron?** – Tauri’s HTML/CSS UI is fine, but for a compression tool users expect **native feel** and tiny footprint (< 10 MB). Pure-Rust UI (Slint/Iced) meets that goal without shipping a browser runtime.

---

#### 8. Open Questions / Input Needed

1. **Archive Formats**: beyond ZIP & TAR.GZ, should we ship 7z, RAR (requires non-OSS libs), zstd-tar, etc.?
2. **Password UX**: enforce minimum length / strength rules?
3. **Split Archives**: support multi-part volumes?
4. **CLI Integration**: bundle a `rolypoly` CLI or keep GUI-only?
5. **Localization Targets**: English only at launch or include others?
6. **Telemetry**: any opt-in crash / usage reporting?

Let me know your preferences here and I’ll lock them into the next revision.
