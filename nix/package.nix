{
  lib,
  stdenv,
  rustPlatform,
  callPackage,
  runCommand,
  zig_0_15,
  zstd,
  pkg-config,
  git,
  cctools ? null,
  xcbuild ? null,
}:

let
  manifest = lib.importTOML ../Cargo.toml;
  cratesIoIndex = "registry+https://github.com/rust-lang/crates.io-index";
  cratesIoNixIndexUrl = "https://herdr.invalid/nix-crates.io-index";
  cratesIoNixIndex = "registry+${cratesIoNixIndexUrl}";
  nixCargoLockContents = builtins.replaceStrings [ cratesIoIndex ] [ cratesIoNixIndex ] (
    builtins.readFile ../Cargo.lock
  );
  zigDeps = callPackage ../vendor/libghostty-vt/build.zig.zon.nix {
    name = "herdr-libghostty-vt-zig-cache";
    inherit zstd;
    linkFarm =
      name: entries:
      runCommand name { } ''
        mkdir -p $out
        ${lib.concatMapStringsSep "\n" (entry: ''
          cp -rL ${entry.path} $out/${entry.name}
        '') entries}
      '';
  };
in
rustPlatform.buildRustPackage {
  pname = "herdr";
  version = manifest.package.version;

  src = lib.fileset.toSource {
    root = ./..;
    fileset = lib.fileset.intersection (lib.fileset.fromSource (lib.sources.cleanSource ./..)) (
      lib.fileset.unions [
        ../assets
        ../docs/next/api/herdr-api.schema.json
        ../src
        ../vendor/libghostty-vt
        ../vendor/libghostty-vt.vendor.json
        ../vendor/portable-pty
        ../build.rs
        ../Cargo.lock
        ../Cargo.toml
        ../CHANGELOG.md
        ../SKILL.md
      ]
    );
  };

  cargoLock = {
    lockFileContents = nixCargoLockContents;

    # Nix needs an alternate download endpoint for crates.io, but mapping the
    # real crates.io index directly makes Cargo define crates-io twice during
    # the vendored build. Use a Nix-only registry alias for fetching instead.
    extraRegistries = {
      "${cratesIoNixIndexUrl}" = "https://static.crates.io/crates";
    };
  };

  postPatch = ''
    substituteInPlace Cargo.lock \
      --replace-fail ${lib.escapeShellArg cratesIoIndex} ${lib.escapeShellArg cratesIoNixIndex}
  '';

  nativeBuildInputs = [
    git
    pkg-config
  ]
  ++ lib.optionals stdenv.hostPlatform.isDarwin [
    cctools
    xcbuild
  ];

  env = {
    LIBGHOSTTY_VT_OPTIMIZE = "ReleaseFast";
    LIBGHOSTTY_VT_SIMD = "true";
    LIBGHOSTTY_VT_ZIG_SYSTEM_DIR = zigDeps;
    ZIG = lib.getExe zig_0_15;
  };

  preBuild = ''
    export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-global-cache"
    export ZIG_LOCAL_CACHE_DIR="$TMPDIR/zig-local-cache"
  '';

  # Rust tests are covered by the normal CI workflow. The Nix check is
  # intentionally build-only so it validates packaging inputs without
  # duplicating the full Rust test suite.
  doCheck = false;

  meta = {
    description = "Terminal workspace manager for AI coding agents";
    homepage = "https://herdr.dev";
    license = lib.licenses.agpl3Plus;
    mainProgram = "herdr";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
}
