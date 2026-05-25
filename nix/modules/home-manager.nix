# nix/modules/home-manager.nix — auto-generated from lava-runtime.caixa.lisp
{ config, lib, pkgs, ... }:
let cfg = config.programs.lava-runtime; in {
  options.programs.lava-runtime = {
    enable = lib.mkEnableOption "lava-runtime";
    package = lib.mkOption { type = lib.types.package; default = pkgs.lava-runtime or null; };
  };
  config = lib.mkIf cfg.enable { home.packages = [ cfg.package ]; };
}
