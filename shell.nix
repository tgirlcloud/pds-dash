{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-tmsZd73cUKgmJROwHYxIMxjOoQ34qmJWQ6X6K+F7V14=";
  },
  pkgs ? import nixpkgs { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    pnpm
    nodejs-slim_24
    typescript-language-server
  ];
}
