{ pkgs, ... }:
{
  packages = with pkgs; [
    git
    rustup
    k6
  ];
}
