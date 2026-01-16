{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-wufp5c0nWh/87f9eK7xy1eZXms5zd4yl6S4SR+LfA08=";
  },
  pkgs ? import nixpkgs { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    pnpm
    nodejs-slim
    typescript-language-server
  ];
}
