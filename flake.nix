{
  description = "A bevy asset loader for Everquest files";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, rust-overlay, crate2nix, ... }:
    let
      name = "bevy_eq_assets";
    in
    utils.lib.eachDefaultSystem
      (system:
        let
          # Imports
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
              (self: super: {
                # Because rust-overlay bundles multiple rust packages into one
                # derivation, specify that mega-bundle here, so that crate2nix
                # will use them automatically.
                rustc = self.rust-bin.nightly.latest.default;
                cargo = self.rust-bin.nightly.latest.default;
              })
            ];
          };
          inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
            generatedCargoNix;

          # Create the cargo2nix project
          project = pkgs.callPackage
            (generatedCargoNix {
              inherit name;
              src = ./.;
            })
            {
              # Individual crate overrides go here
              # Example: https://github.com/balsoft/simple-osd-daemons/blob/6f85144934c0c1382c7a4d3a2bbb80106776e270/flake.nix#L28-L50
              defaultCrateOverrides = pkgs.defaultCrateOverrides // {
                # The app crate itself is overriden here. Typically we
                # configure non-Rust dependencies (see below) here.
                ${name} = oldAttrs: {
                  inherit buildInputs nativeBuildInputs;
                } // buildEnvVars;
              };
            };

          # Configuration for the non-Rust dependencies
          buildInputs = with pkgs; [
            udev
            alsa-lib
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr # To use the x11 feature
            libxkbcommon
            wayland # To use the wayland feature
          ];
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
            nixpkgs-fmt
            rust-analyzer
            perf-tools
            cargo-flamegraph
            cargo-readme
          ];
          buildEnvVars = { };
        in
        rec {
          packages.${name} = project.build;

          # `nix build`
          defaultPackage = packages.${name};

          # `nix run`
          apps.${name} = utils.lib.mkApp {
            inherit name;
            drv = packages.${name};
          };
          defaultApp = apps.${name};

          # `nix develop`
          devShell = pkgs.mkShell
            {
              inherit buildInputs nativeBuildInputs;
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
              LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath buildInputs;
            } // buildEnvVars;
        }
      );
}
