# nix/modules/nixos.nix — auto-generated from lava-runtime.caixa.lisp
# description: "Unified EmbeddedRuntime trait for in-process IaC DSL evaluation. Wraps lava-eval (tatara-lisp), pangea-ruby-eval (Ruby/magnus), and (future) tatara-script — magma consumes any runtime via one typed surface. One orchestration shape for all DSLs; zero shell-out, zero IPC, zero disk-roundtrip between authoring and apply."
{ config, lib, pkgs, ... }:
let
  cfg = config.services.lava-runtime;
in {
  options.services.lava-runtime = {
    enable = lib.mkEnableOption "lava-runtime";
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.lava-runtime or null;
    };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
