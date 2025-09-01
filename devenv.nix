{ pkgs, ... }:
{
  packages = with pkgs; [
    git
    rustup
    cargo-nextest
    cargo-insta
    k6
    nodejs
    taplo
    jq
  ];

  enterShell = ''
    export PATH="$DEVENV_ROOT/node_modules/.bin:$PATH"
  '';
}
