# RolyPoly — Modern ZIP CLI

Fast, safe ZIP archiver written in Rust with a clean CLI.

**Highlights**
- Cross‑platform: Linux, macOS, Windows
- Integrity checks: CRC32 and SHA‑256
- Progress output (human and JSON)
- List, extract, create, stats, hash

**Install**
- From source: `cargo install --path .` or `cargo build --release --bin rolypoly`
- Binaries: see GitHub Releases once available

**Usage**
- Create: `rolypoly create archive.zip file1 dir/`
- Extract: `rolypoly extract archive.zip -o out/`
- List: `rolypoly list archive.zip`
- Validate: `rolypoly validate archive.zip`
- Stats: `rolypoly stats archive.zip`
- Hash: `rolypoly hash file.txt`

**Testing**
- Quick check: `./dev test` (fmt + clippy + tests)
- Tests only: `./dev test:quick` or `cargo test --all --no-default-features`

**Release**
- Tag: `./scripts/release.sh cli 0.1.0`
- Push: `git push origin release/cli && git push origin cli-v0.1.0`
- GitHub Actions will build artifacts and create a Release with a changelog

**GUI (Flutter, optional)**
- Code lives in `gui/` and shells out to the CLI.
- Dev quickstart: `./scripts/gui_dev.sh` (requires Flutter installed)
- Details: see `gui/README.md`
