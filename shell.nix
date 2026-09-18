{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-j08lBbqaYwjsW0xy6Dh4MNZpQ/oSvLU1MLAo38OXphM=";
  },
  pkgs ? import nixpkgs { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    rust-analyzer
    rustfmt
    clippy
    pkg-config
    openssl
  ];
}
