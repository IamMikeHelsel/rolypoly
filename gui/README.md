RolyPoly GUI (Flutter)

Overview
- Desktop Flutter app providing a clean, modern UI for the RolyPoly CLI.
- Initial integration uses a process bridge: the app spawns `rolypoly` and parses output.

Getting Started
1) Install Flutter (stable) and enable desktop targets:
   - flutter config --enable-macos-desktop --enable-windows-desktop --enable-linux-desktop
2) Bootstrap the project:
   - cd gui
   - flutter create .
   - flutter pub add hooks_riverpod go_router flex_color_scheme flutter_animate animations
3) Run the app:
   - flutter run -d macos|windows|linux

App Structure (planned)
- lib/main.dart: Entry point + theme.
- lib/router.dart: Routes using go_router.
- lib/theme.dart: FlexColorScheme setup (light/dark).
- lib/services/rolypoly_cli.dart: Process bridge to CLI with progress parsing.
- lib/features/
  - compress.dart: Add files, list, create archive with progress.
  - extract.dart: Pick archive, choose destination, extract with progress.
  - inspect.dart: List contents, filter, basic props.
  - validate_stats.dart: Validate and show stats.

Notes
- Ensure `rolypoly` is on PATH, or set an absolute path in the app (to be added in Settings).
- For releases, consider bundling the CLI next to the app binary; the GUI can detect and run it.

