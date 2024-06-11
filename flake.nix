{
  description = "A devShell that can run three-d examples";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        rust = pkgs.rust-bin.nightly.latest.default.override {
          targets = ["wasm32-unknown-unknown"];
        };

        libs = with pkgs; [
          alsaLib
          udev
          vulkan-loader
          wayland
          libxkbcommon
        ];
        buildDeps = with pkgs;
          [
            pkg-config
            rust
            lld
            clang
            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            wasm-bindgen-cli
          ]
          ++ libs;

        utils = with pkgs; [
          #  video driver info
          pciutils
          glxinfo
          gdb
          lldb
          rust-analyzer
        ];
        LIBS = pkgs.lib.makeLibraryPath libs;
        run = pkgs.writeShellScriptBin "run" ''
          export LD_LIBRARY_PATH=${LIBS}
          ${rust}/bin/cargo run $@
        '';
      in
        with pkgs; {
          devShells.default = mkShell {
            inherit LIBS;
            name = "rust graphics env";
            WINIT_UNIX_BACKEND = "wayland";
            buildInputs = buildDeps ++ utils ++ [run];
            shellHook = ''
              echo Entering rust env!
            '';
          };
        }
    );
}
