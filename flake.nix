{
  description = "Rust web-crawler and indexer development shell with Fenix";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    fenix.url        = "github:nix-community/fenix";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default rust-overlay.overlays.default ];
        };
        lib = pkgs.lib;

        # Use Fenix's complete stable toolchain and latest rust-analyzer
        stableToolchain = fenix.packages.${system}.complete.toolchain;
        rustAnalyzer    = fenix.packages.${system}.latest.rust-analyzer;
      in {
        devShells.default = pkgs.mkShell {
          name = "rust-web-crawler-shell";

          buildInputs = with pkgs; lib.flatten [
            stableToolchain
            rustAnalyzer
            cargo-expand
            jq
            pkg-config
            openssl
            firefox
            geckodriver
          ];

          shellHook = ''
            echo "Using Rust toolchain: $(rustc --version)"
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export PKG_CONFIG_PATH=${pkgs.pkg-config}/lib/pkgconfig
            export RUST_BACKTRACE=1
            # Ensure local cargo cache in home dir
            export CARGO_HOME="$HOME/.cargo"
            # Avoid accidental writes to Nix store; RUSTUP_HOME not used by Fenix
            export RUSTUP_HOME="$HOME/.rustup"
            mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"

            if ! pgrep -x geckodriver > /dev/null; then
              geckodriver > geckodriver.log 2>&1 &
              trap "kill $!" EXIT
            fi

            exec nu --login
          '';
        };
      }
    );
}
