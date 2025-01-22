{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk { };
        bacon = naersk'.buildPackage {
          buildInputs = [ pkgs.alsa-lib pkgs.pkg-config ];
          src = ./.;
        };
      in
      {
        # For `nix build` & `nix run`:
        defaultPackage = bacon;

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
          ];
        };

        # Overlay for package usage in other Nix configurations
        overlay = final: prev: {
          bacon = bacon;
        };
      }
    );
}
