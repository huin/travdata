{ pkgs, lib, config, inputs, ... }:

let
  jdk = pkgs.jdk23_headless;
in
{
  # https://devenv.sh/basics/
  env.GREET = "devenv";
  env.JDK = builtins.toString "${jdk.out}/lib/openjdk/lib/server";
  env.GSETTINGS_SCHEMA_DIR = "${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}/glib-2.0/schemas/";

  # https://devenv.sh/packages/
  packages = [
    pkgs.git
    pkgs.gtk4.dev
    pkgs.pkg-config
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
