{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-Tf7q8/0aIwg4Btd/GUMNHXDTqlrWTr5o/vZ8euFUAQY=";
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
