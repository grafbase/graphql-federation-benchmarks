{pkgs, ...}: {
  packages = with pkgs; [
    git
    rustup
    cargo-nextest
    cargo-insta
    k6
  ];
}
