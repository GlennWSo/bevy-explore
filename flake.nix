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
        wasmArtifacts = craneLib.buildDepsOnly {
          inherit src buildInputs;
          version = "0.1.0";

          cargoExtraArgs = "--target wasm32-unknown-unknown";
          doCheck = false;
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
          inherit src buildInputs;
          cargoArtifacts = wasmArtifacts;
          cargoExtraArgs = "--target wasm32-unknown-unknown";
          doCheck = false;
          postFixup = ''
            mkdir $out/bin/wasm
            cp -r assets $out/bin/wasm/

            cd $out/bin
            cp ${./www/index.html} index.html
            cp ${./www/wasm/index.html} wasm/index.html

            mv ${wasmBuild.pname}.wasm wasm/
            wasm-bindgen --no-typescript --target web \
              --out-dir wasm/ \
              --out-name game \
              wasm/${wasmBuild.pname}.wasm
          '';
        };
        wasmPublish = pkgs.writeShellScriptBin "publish" ''
          rm -rf docs/*
          mkdir docs/wasm/assets -p
          install result/bin/index.html docs/
          install $(find result/bin/wasm/ -maxdepth 1 -type f ) docs/wasm/
          install $(find result/bin/wasm/assets -maxdepth 1 -type f ) docs/wasm/assets/
        '';
        wasmServe = with pkgs;
          writeShellScriptBin "serve" ''
            ${static-server}/bin/static-server $@ ${wasmBuild}/bin/
          '';
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
          vscode-langservers-extracted
          # wasmServe
          wasmPublish
          static-server
        ];
        LIBS = pkgs.lib.makeLibraryPath libs;
        run = pkgs.writeShellScriptBin "run" ''
          export LD_LIBRARY_PATH=${LIBS}
          ${rust}/bin/cargo run $@
        '';
      in
        with pkgs; {
          packages = {
            inherit wasmBuild wasmArtifacts wasmServe;
            default = nixCrate;
            deps = cargoArtifacts;
          };
          devShells.default = mkShell {
            inherit LIBS;
            name = "rust graphics env";
            buildInputs = buildInputs ++ utils ++ [run];

            RUST_LOG = "error";
            WINIT_UNIX_BACKEND = "wayland";
            shellHook = ''
              echo Entering rust env!
              echo log level [RUST_LOG]: $RUST_LOG
            '';
          };
        }
    );
}
