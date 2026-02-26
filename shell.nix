{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-lJiQtL47xYV37Vod9KgPS7dmi1x51A5+chJE48qKimw=";
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
