{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-luOcHMj74mEbDfB9dbcvMlXUF/6oefoKI2xxyZAvyy0=";
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
