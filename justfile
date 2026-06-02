
test:
    cargo nextest run --success-output never --failure-output immediate --final-status-level slow

precheck:
    cargo test --no-default-features --features contextual --verbose
    cargo clippy --all-features --all-targets -- -D warnings
    just test

doc:
    cargo doc --all-features --no-deps
