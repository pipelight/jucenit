{
  config,
  pkgs,
  lib,
  inputs,
  ...
}: {
  users.groups = {
    unit.members = "username";
  };
  ## Add Nginx unit global package and systemd unit.
  services.unit.enable = true;

  ## Custom systemd unit.
  systemd.services.unit = {
    enable = true;
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      # Replaces default unix socket with local tcp port
      ExecStart = lib.mkForce ''
        ${pkgs.unit}/bin/unitd \
          --control '127.0.0.1:8080' \
          --pid '/run/unit/unit.pid' \
          --log '/var/log/unit/unit.log' \
          --statedir '/var/spool/unit' \
          --tmpdir '/tmp' \
          --user unit \
          --group unit
      '';
      # Disable configuration deletion across reboot
      ExecStartPost = lib.mkForce "";
    };
  };
}
