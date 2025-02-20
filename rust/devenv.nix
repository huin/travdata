{ pkgs, lib, config, inputs, ... }:

let
  jdk = pkgs.jdk23_headless;
in
{
  # https://devenv.sh/basics/
  env = {
    GREET = "devenv";
    JDK = builtins.toString "${jdk.out}/lib/openjdk/lib/server";
    GSETTINGS_SCHEMA_DIR = "${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}/glib-2.0/schemas/";
    PDFIUM_DYNAMIC_LIB_PATH = "${pkgs.pdfium-binaries}/lib";
  };

  # https://devenv.sh/packages/
  packages = with pkgs; [
    git
    gtk4.debug
    gtk4.dev
    gtk4.devdoc
    pkg-config
    pdfium-binaries
  ];

  # https://devenv.sh/scripts/
  scripts.hello.exec = "echo hello from $GREET";

  enterShell = ''
    hello
    git --version
  '';

  # https://devenv.sh/tests/
  # enterTest = ''
  #   echo "Running tests"
  #   git --version | grep "2.42.0"
  # '';

  # https://devenv.sh/languages/
  languages.c.enable = true;
  languages.java.enable = true;
  languages.java.jdk.package = jdk;
  languages.java.maven.enable = true;
  languages.rust.enable = true;
  languages.rust.channel = "stable";

  # https://devenv.sh/pre-commit-hooks/
  # pre-commit.hooks.shellcheck.enable = true;

  # https://devenv.sh/processes/
  # processes.ping.exec = "ping example.com";

  # See full reference at https://devenv.sh/reference/options/
}
