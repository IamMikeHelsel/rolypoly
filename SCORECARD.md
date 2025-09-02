# RolyPoly Release Scorecard

## Test Summary
```
warning: unexpected `cfg` condition value: `gui`
 --> tests/gui_integration_tests.rs:1:8
  |
1 | #![cfg(feature = "gui")]
  |        ^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `gui` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default

warning: unexpected `cfg` condition value: `gui`
 --> tests/gui_ui_tests.rs:1:8
  |
1 | #![cfg(feature = "gui")] // No-op: GUI removed; keeps file ignored
  |        ^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `gui` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default

warning: unexpected `cfg` condition value: `gui`
 --> tests/e2e_tests.rs:1:8
  |
1 | #![cfg(feature = "gui")] // No-op now that GUI is removed
  |        ^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `gui` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default

warning: unexpected `cfg` condition value: `gui`
 --> tests/gui_component_tests.rs:1:8
  |
1 | #![cfg(feature = "gui")]
  |        ^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `gui` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default


running 21 tests
.....................
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 5 tests
.....
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.55s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 2 tests
i.
test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Performance (Synthetic)

| Operation | Data (MB) | Time (ms) | MB/s |
|-----------|-----------|-----------|------|
| Create    | 2       | 23      | 86.96 |
| Extract   | 2       | 23      | 86.96 |

Generated at 2025-09-02T21:34:13Z
