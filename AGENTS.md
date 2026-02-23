# Testing Strategy

This document outlines the testing strategy for the RolyPoly project.

## CLI Testing

The CLI is tested using a combination of unit tests and integration tests.

*   **Unit Tests**: Located in source files (e.g., `src/cli.rs`, `src/archive.rs`).
*   **Integration Tests**: Located in `tests/`. These tests run the built binary against test scenarios.

To run CLI tests:
```bash
cargo test
```

## GUI Testing

The GUI (Flutter) interacts with the CLI as a backend. To ensure this interaction is robust, we have a comprehensive testing suite.

*   **Component Tests**: Located in `tests/gui_component_tests.rs`. These tests verify the specific CLI commands and flags that the GUI relies on. They are gated by the `gui` feature.
*   **Comprehensive Test Script**: `test_gui_comprehensive.py` orchestrates the testing process. It builds the project, runs the specific component tests, and simulates the GUI's expectations.

To run the comprehensive GUI tests:
```bash
python3 test_gui_comprehensive.py
```

This script will:
1.  Check Rust compilation.
2.  Run backend component tests (checking health, create, list, validate, stats, hash, etc.).
3.  Verify error handling and message formatting.
4.  Test concurrent operations.
5.  Verify the complete workflow.

## Release Testing

Before release, ensure:
1.  All cargo tests pass: `cargo test --all-features`
2.  The comprehensive GUI test script passes: `python3 test_gui_comprehensive.py`
3.  Manual verification of the GUI (if possible on the target platform).
