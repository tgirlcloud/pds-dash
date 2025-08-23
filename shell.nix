# girl who's shell isn't reproducible
{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    pnpm
    nodejs-slim_24
    typescript-language-server
  ];
}
