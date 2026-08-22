{
  description = "Pinhole development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    # x86_64-darwin is excluded: this nixpkgs pin has dropped support for it.
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        isDarwin = pkgs.stdenv.hostPlatform.isDarwin;

        # Runtime libraries needed by iced (wgpu/winit) at run time on Linux —
        # on Darwin, windowing/GPU access goes through system frameworks
        # instead (see darwinFrameworks below), so this is empty there.
        runtimeLibs = with pkgs; pkgs.lib.optionals (!isDarwin) [
          vulkan-loader
          libGL
          wayland
          libxkbcommon
          libx11
          libxcursor
          libxi
          libxrandr
          fontconfig
          freetype
        ];

        # SDK iced/winit/wgpu need to build and link against on macOS (Cocoa,
        # Metal, etc.) — per-framework packages (`darwin.apple_sdk.frameworks.*`)
        # were removed from nixpkgs; `apple-sdk` is the current replacement.
        darwinFrameworks = pkgs.lib.optionals isDarwin [ pkgs.apple-sdk ];

        # Builds every workspace member at once (bin crates share a lockfile
        # and most of their dependency graph, so there's no benefit to
        # building them separately). Individual `packages.<name>` below just
        # extract one binary each from this shared build.
        workspace = pkgs.rustPlatform.buildRustPackage {
          pname = "pinhole-workspace";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = runtimeLibs ++ darwinFrameworks;

          # Integration tests need TLS certs and open sockets/windows; the
          # existing CI job runs `cargo test` directly instead.
          doCheck = false;

          installPhase = ''
            runHook preInstall
            mkdir -p $out/bin
            install -Dm755 target/*/release/pinhole-client $out/bin/ 2>/dev/null || \
              install -Dm755 target/release/pinhole-client $out/bin/
            install -Dm755 target/*/release/pinhole-todomvc-example $out/bin/ 2>/dev/null || \
              install -Dm755 target/release/pinhole-todomvc-example $out/bin/
            install -Dm755 target/*/release/pinhole-styling-showcase-example $out/bin/ 2>/dev/null || \
              install -Dm755 target/release/pinhole-styling-showcase-example $out/bin/
            runHook postInstall
          '';
        };

        # Extract a single binary from the shared workspace build.
        # pname/version (not just name) are set because `nix bundle`'s
        # default bundler requires them.
        mkBinPackage = name: pkgs.runCommand name { pname = name; version = "0.1.0"; } ''
          mkdir -p $out/bin
          ln -s ${workspace}/bin/${name} $out/bin/${name}
        '';

        # The GUI client needs its Vulkan/Wayland/X11 runtime libs on
        # LD_LIBRARY_PATH to run outside the devShell (which sets this
        # interactively) — wrap it rather than relying on the caller's
        # environment.
        pinhole-client-linux = pkgs.symlinkJoin {
          name = "pinhole-client";
          pname = "pinhole-client";
          version = "0.1.0";
          paths = [ (mkBinPackage "pinhole-client") ];
          buildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/pinhole-client \
              --set LD_LIBRARY_PATH ${pkgs.lib.makeLibraryPath runtimeLibs}
          '';
        };

        # A double-clickable .app bundle for macOS, since that's the native
        # packaging format there rather than a bare CLI binary. Also
        # symlinks $out/bin/pinhole-client to the bundle's executable so
        # `nix run .#pinhole-client` still works.
        #
        # NOT independently verified on real macOS hardware — this repo is
        # developed on Linux and Nix can't cross-build a macOS GUI binary
        # (it needs Apple's SDK frameworks, only available when building on
        # Darwin itself). `nix flake check` confirms the derivation is
        # well-formed, but someone needs to actually `nix build
        # .#pinhole-client` on an aarch64-darwin machine to confirm it runs.
        pinhole-client-darwin = pkgs.stdenv.mkDerivation {
          pname = "pinhole-client-app";
          version = "0.1.0";
          dontUnpack = true;

          installPhase = ''
            runHook preInstall

            app="$out/Applications/Pinhole.app"
            mkdir -p "$app/Contents/MacOS"
            cp ${workspace}/bin/pinhole-client "$app/Contents/MacOS/Pinhole"

            cat > "$app/Contents/Info.plist" <<EOF
            <?xml version="1.0" encoding="UTF-8"?>
            <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
            <plist version="1.0">
            <dict>
              <key>CFBundleName</key><string>Pinhole</string>
              <key>CFBundleDisplayName</key><string>Pinhole</string>
              <key>CFBundleIdentifier</key><string>net.michaelmelanson.pinhole</string>
              <key>CFBundleVersion</key><string>0.1.0</string>
              <key>CFBundleShortVersionString</key><string>0.1.0</string>
              <key>CFBundleExecutable</key><string>Pinhole</string>
              <key>CFBundlePackageType</key><string>APPL</string>
              <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
              <key>NSHighResolutionCapable</key><true/>
            </dict>
            </plist>
            EOF

            mkdir -p $out/bin
            ln -s "$app/Contents/MacOS/Pinhole" $out/bin/pinhole-client

            runHook postInstall
          '';
        };
      in
      {
        packages = {
          default = workspace;

          pinhole-client = if isDarwin then pinhole-client-darwin else pinhole-client-linux;

          pinhole-todomvc-example = mkBinPackage "pinhole-todomvc-example";
          pinhole-styling-showcase-example =
            mkBinPackage "pinhole-styling-showcase-example";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer

            cargo-watch
            cargo-outdated
          ] ++ runtimeLibs ++ darwinFrameworks;

          LD_LIBRARY_PATH = pkgs.lib.optionalString (!isDarwin) (pkgs.lib.makeLibraryPath runtimeLibs);

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      });
}
