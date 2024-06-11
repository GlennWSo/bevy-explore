{
  description = "A devShell that can run three-d examples";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    crane,
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
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;
        fs = pkgs.lib.fileset;
        files = fs.unions [
          ./.cargo
          ./src
          ./assets
          (fs.fileFilter (file: file.hasExt "toml") ./.)
          (fs.fileFilter (file: file.name == "Cargo.lock") ./.)
        ];

        src = fs.toSource {
          root = ./.;
          fileset = files;
        };

        cargoArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
          version = "0.1.0";
          doCheck = false;

          # cargoExtraArgs = "--target thumbv7em-none-eabihf";
        };

        nixCrate = craneLib.buildPackage {
          inherit src cargoArtifacts buildInputs;
          postFixup = ''
            cp -r assets $out/bin/
            wrapProgram $out/bin/${nixCrate.pname} \
              --set LD_LIBRARY_PATH ${LIBS}
          '';
        };

        wasmBuild = craneLib.buildPackage {
          inherit src cargoArtifacts buildInputs;

          cargoExtraArgs = "--target wasm32-unknown-unknown";
          doCheck = false;
          postFixup = ''
            cp ${./www/index.html} $out/bin/index.html
            cp -r assets $out/bin/
            wasm-bindgen --no-typescript --target web \
              --out-dir $out/bin/ \
              --out-name game \
              $out/bin/${wasmBuild.pname}.wasm
          '';
        };
        libs = with pkgs; [
          alsaLib
          udev
          vulkan-loader
          wayland
          libxkbcommon
        ];
        buildInputs = with pkgs;
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
            makeWrapper
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
          packages = {
            inherit wasmBuild;
            default = nixCrate;
            deps = cargoArtifacts;
          };
          devShells.default = mkShell {
            inherit LIBS;
            name = "rust graphics env";
            WINIT_UNIX_BACKEND = "wayland";
            buildInputs = buildInputs ++ utils ++ [run];
            shellHook = ''
              echo Entering rust env!
            '';
          };
        }
    );
}
