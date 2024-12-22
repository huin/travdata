{ pkgs, lib, config, inputs, ... }:

let
  extra_libs = with pkgs; [
    stdenv.cc.cc.lib

    # Needed for the Python Qt based GUI:
    dbus.lib
    fontconfig.lib
    freetype.out
    glib.out
    libsForQt5.qt5.qtwayland
    libglvnd.out
    libxkbcommon.out
    wayland
    xorg.libX11.out
    zstd.out
  ];
  jdk = pkgs.jdk23_headless;
in

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.EXTRA_LIBS = builtins.toString extra_libs;
  env.JDK = builtins.toString jdk;
  env.LD_LIBRARY_PATH =
    builtins.concatStringsSep ":" (
      builtins.concatLists [
        [(jdk + "/lib/server")]
        (builtins.map (path: path + "/lib") extra_libs)
      ]
    );

  # https://devenv.sh/packages/
  packages = with pkgs; [
    gdbHostCpuOnly
    git
    jdk
    poetry
    pre-commit
    python311Full
  ];

  # https://devenv.sh/scripts/
  scripts.hello.exec = "echo hello from $GREET";

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/tests/
  enterTest = ''
    echo "Running tests"
    git --version | grep "2.42.0"
  '';

  # See full reference at https://devenv.sh/reference/options/
}
