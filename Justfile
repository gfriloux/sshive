run:
    cargo run

build:
    cargo build --release

test:
    cargo test

fmt:
    cargo fmt

check:
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo test
    cargo audit

third-party-licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES

nix-check:
    nix flake check
