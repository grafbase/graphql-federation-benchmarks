{pkgs, ...}: {
  packages = with pkgs; [
    git
    rustup
  ];
}
