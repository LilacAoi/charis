{
  description = "Charis development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "charis-dev-shell";
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
          ];
          buildInputs = with pkgs; [
            webkitgtk_4_1
            gtk3
            libsoup_3
            glib
          ];
        };
      }
    );
}
