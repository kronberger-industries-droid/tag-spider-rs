{ description = "dropkitten helper for Nix flakes with basic Rust dev shell";

inputs = {
  nixpkgs.url     = "github:NixOS/nixpkgs/nixos-24.11";
  flake-utils.url = "github:numtide/flake-utils";
};

outputs = {nixpkgs, flake-utils, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells = {
        default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # rust development
            rustup
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer

            # tools
            nushell
            cargo-expand

            # drivers
            firefox
            geckodriver

            # dependencies
            openssl
            pkg-config
          ];

          shellHook = ''
            # rust toolchain init
            if ! rustup toolchain list | grep -q stable; then
              rustup toolchain install stable
            fi
            rustup default stable

            if ! pgrep -x geckodriver >/dev/null; then
              geckodriver > geckodriver.log 2>&1 &
              GECKODRIVER_PID=$!
              # only kill the one we launched
              trap "kill $GECKODRIVER_PID" EXIT
            fi

            # start into nushell
            nu
          '';
        };
      };
    }
  );
}
