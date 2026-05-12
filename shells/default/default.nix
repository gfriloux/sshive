{
  pkgs,
  mkShell,
  ...
}:
mkShell {
  packages = with pkgs; [
    # Nix
    alejandra
    deadnix
    statix
    pre-commit

    # Rust
    cargo
    rustc
    clippy
    rustfmt
    rust-analyzer
    cargo-audit
    cargo-deny
    cargo-about

    # Tauri CLI + Node (app + docs)
    cargo-tauri
    nodejs_22

    # Build tooling
    pkg-config

    # Outils dev
    git-cliff
    just

    # ssh-keygen (validation fingerprints en dev)
    openssh
  ];

  # Tauri v2 — stack GTK/WebKit requise sur Linux
  buildInputs = with pkgs; [
    dbus
    openssl
    glib
    glib-networking
    gtk3
    webkitgtk_4_1
    libsoup_3
    cairo
    pango
    gdk-pixbuf
    atk
    librsvg
    xdotool
  ];

  shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath (with pkgs; [
      dbus
      openssl
      glib
      gtk3
      webkitgtk_4_1
      libsoup_3
      cairo
      pango
      gdk-pixbuf
      atk
      librsvg
    ])}:$LD_LIBRARY_PATH

        echo "[sshive] Ready. Run: cd tauri-app && cargo-tauri dev"

        if [ ! -f .pre-commit-config.yaml ]; then
          echo "Generating .pre-commit-config.yaml..."
          cat > .pre-commit-config.yaml <<'EOF'
    ---
    repos:
      - repo: local
        hooks:
          - id: alejandra
            name: alejandra
            language: system
            entry: alejandra --check
            files: \.nix$
            pass_filenames: true
          - id: deadnix
            name: deadnix
            language: system
            entry: deadnix --fail
            files: \.nix$
            pass_filenames: false
          - id: statix
            name: statix
            language: system
            entry: statix check
            files: \.nix$
            pass_filenames: false
          - id: rustfmt
            name: rustfmt
            language: system
            entry: cargo fmt -- --check --config tab_spaces=2
            files: \.rs$
            pass_filenames: false
          - id: clippy
            name: clippy
            language: system
            entry: cargo clippy -- -D warnings
            files: \.rs$
            pass_filenames: false
    EOF
        else
          echo ".pre-commit-config.yaml already exists. Skipping generation."
        fi

        if [ -d .git ]; then
          if [ ! -f .git/hooks/pre-commit ]; then
            echo "Installing pre-commit hook..."
            pre-commit install -f --install-hooks
          fi
        else
          echo "Not a git repository. Skipping pre-commit installation."
        fi
  '';
}
