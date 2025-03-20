{ pkgs }:

{
  buildInputs = with pkgs; [
    firefox
    geckodriver
    openssl
    pkg-config
    cargo-expand
  ];
  shellHook = ''
    # In your shellHook:
    geckodriver > geckodriver.log 2>&1 &
    GECKODRIVER_PID=$!
    trap "kill $GECKODRIVER_PID" EXIT
    nu
  '';
}
