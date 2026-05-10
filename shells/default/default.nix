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

    # Outils dev
    git-cliff
    just

    # iced (wayland + x11)
    wayland
    libxkbcommon
    libGL
    vulkan-loader

    # rfd (native file dialog — v0.2+)
    gtk3

    # ssh-keygen (validation fingerprints en dev)
    openssh
  ];

  shellHook = ''
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.wayland
      pkgs.libxkbcommon
      pkgs.libGL
      pkgs.vulkan-loader
      pkgs.dbus
    ]}:$LD_LIBRARY_PATH

        echo "[sshive] Ready."

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
            pass_filenames: true
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
