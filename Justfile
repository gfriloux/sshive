dev:
    cd tauri-app && cargo-tauri dev

build:
    cd tauri-app && cargo-tauri build

# AppImage nécessite nix-ld (ou un système non-NixOS).
# Tauri télécharge linuxdeploy dynamiquement — incompatible sans /lib64/ld-linux-x86-64.so.2.
# Pour activer : ajouter `programs.nix-ld.enable = true;` dans ta config NixOS système.
build-appimage:
    cd tauri-app && cargo-tauri build --bundles appimage

build-bin:
    cd tauri-app && cargo-tauri build --no-bundle

run:
    ./tauri-app/src-tauri/target/release/sshive-app

test:
    cargo test

fmt:
    cargo fmt -- --config tab_spaces=2
    cd tauri-app && cargo fmt --manifest-path src-tauri/Cargo.toml -- --config tab_spaces=2

check:
    cargo fmt --check -- --config tab_spaces=2
    cargo clippy -- -D warnings
    cargo test
    # RUSTSEC-2023-0071 : Marvin Attack dans rsa via ssh-key.
    # SSHive n'effectue pas d'opérations RSA (ed25519 uniquement). Aucun fix upstream disponible.
    cargo audit --ignore RUSTSEC-2023-0071

third-party-licenses:
    cargo about generate about.hbs -o THIRD_PARTY_LICENSES

docs-dev:
    cd docs && npm run dev

docs-build:
    cd docs && npm run build

docs-install:
    cd docs && npm install

screenshots:
    cd tauri-app && npm install
    node scripts/screenshots.mjs

nix-check:
    nix flake check
