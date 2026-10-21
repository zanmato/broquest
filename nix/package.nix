{
  lib,
  craneLib,
  llrt,
  runCommand,
  pkg-config,
  alsa-lib,
  fontconfig,
  freetype,
  libxkbcommon,
  libx11,
  libxcb,
  libxcursor,
  libxi,
  libxrandr,
  openssl,
  vulkan-loader,
  wayland,
  zlib,
}:

let
  root = ../.;

  # Sources the build reads. Docs, the website and the Nix files stay out, so
  # editing them does not rebuild Broquest.
  ownSource = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
    ];
  };

  src = runCommand "broquest-source" { } ''
    cp -r ${ownSource} $out
    chmod u+w $out
    mkdir $out/vendor
    cp -r ${llrt} $out/vendor/llrt
  '';

  cargoToml = lib.importTOML ../Cargo.toml;

  # GPUI loads these with dlopen, so the linker never records them. They go in
  # an rpath rather than a wrapper's LD_LIBRARY_PATH, which every program
  # Broquest starts would inherit.
  runtimeLibraries = [
    vulkan-loader
    wayland
    libxkbcommon
    libx11
    libxcb
    libxcursor
    libxi
    libxrandr
  ];

  commonArgs = {
    inherit src;
    pname = "broquest";
    version = cargoToml.workspace.package.version;
    strictDeps = true;
    cargoExtraArgs = "--package broquest";

    nativeBuildInputs = [ pkg-config ];

    buildInputs = [
      alsa-lib
      fontconfig
      freetype
      openssl
      zlib
    ]
    ++ runtimeLibraries;
  };

  # Every dependency built once and reused until Cargo.lock changes.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
craneLib.buildPackage (
  commonArgs
  // {
    inherit cargoArtifacts;

    # The tests start servers and browsers, so they run in CI instead.
    doCheck = false;

    NIX_LDFLAGS = "-rpath ${lib.makeLibraryPath runtimeLibraries}";
    # The fixup's patchelf --shrink-rpath keeps only directories of libraries
    # the binary links, which would drop the ones above.
    dontPatchELF = true;

    passthru = { inherit runtimeLibraries; };

    postInstall = ''
      install -Dm644 crates/broquest/resources/broquest.desktop \
        $out/share/applications/broquest.desktop
      install -Dm644 crates/broquest/resources/broquest-icon-512x512.png \
        $out/share/icons/hicolor/512x512/apps/broquest.png
    '';

    meta = {
      description = cargoToml.workspace.package.description;
      homepage = "https://github.com/zanmato/broquest";
      license = lib.licenses.asl20;
      mainProgram = "broquest";
      platforms = lib.platforms.linux;
    };
  }
)
