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
    # RUSTSEC-2023-0071 : Marvin Attack dans rsa via ssh-key.
    # SSHive n'effectue pas d'opérations RSA (ed25519 uniquement). Aucun fix upstream disponible.
    cargo audit --ignore RUSTSEC-2023-0071

third-party-licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES

nix-check:
    nix flake check
