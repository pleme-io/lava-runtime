# nix/modules/darwin.nix — auto-generated from lava-runtime.caixa.lisp
{ config, lib, pkgs, ... }:
let cfg = config.services.lava-runtime; in {
  options.services.lava-runtime = {
    enable = lib.mkEnableOption "lava-runtime";
    package = lib.mkOption { type = lib.types.package; default = pkgs.lava-runtime or null; };
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}
