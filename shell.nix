{
  nixpkgs ? builtins.fetchTarball {
    url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.xz";
    sha256 = "sha256-mU85VDFRIgKGq1EhT71bLjhvjJ5yuMEe0Ip1kwCbR80=";
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
