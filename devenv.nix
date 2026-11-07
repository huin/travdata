{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  extra_libs = with pkgs; [
    stdenv.cc.cc.lib

    # Needed for the Python Qt based GUI:
    dbus.lib
    fontconfig.lib
    freetype.out
    glib.out
    qt5.qtwayland
    libglvnd.out
    libxkbcommon.out
    wayland
    libX11.out
    zstd.out
  ];
  jdk = pkgs.jdk25_headless;
in

{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.EXTRA_LIBS = builtins.toString extra_libs;
  env.JDK = builtins.toString jdk;
  env.LD_LIBRARY_PATH = builtins.concatStringsSep ":" (
    builtins.concatLists [
      [ (jdk + "/lib/server") ]
      (builtins.map (path: path + "/lib") extra_libs)
    ]
  );

  # https://devenv.sh/packages/
  packages = with pkgs; [
    gdbHostCpuOnly
    git
    jdk
    pre-commit
  ];

  languages.python = {
    enable = true;
    version = "3.11.17";
    uv = {
      enable = true;
      sync.enable = true;
      sync.allGroups = true;
    };
  };

  # https://devenv.sh/scripts/

  # https://devenv.sh/tests/

  # See full reference at https://devenv.sh/reference/options/
}
