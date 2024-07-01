{
  config,
  pkgs,
  lib,
  inputs,
  ...
}: let
  cfg = {
    # Package configuration variables
    user = "unit";
    group = "unit";
    stateDir = "/var/spool/unit";
    logDir = "/var/log/unit";
    challengDir = "/tmp/jucenit";
  };
in {
  users.users.${cfg.user} = {
    group = "${cfg.group}";
    isSystemUser = true;
  };
  users.groups = {
    unit.members = [
      # Add users that can manage unit/jucenit files
      "anon"
    ];
  };

  environment.defaultPackages = with pkgs; let
    # Get package from flake inputs
    jucenit = inputs.jucenit.packages.${system}.default;
  in [
    # Web server and dependencies
    jucenit
    unit
  ];

  systemd.tmpfiles.rules = [
    # Nginx-unit file permissions (bit mode)
    "d '${cfg.stateDir}' 0750 ${cfg.user} ${cfg.group} - -"
    "Z '${cfg.stateDir}' 0750 ${cfg.user} ${cfg.group} - -"
    "d '${cfg.logDir}' 0750 ${cfg.user} ${cfg.group} - -"
    "Z '${cfg.logDir}' 0750 ${cfg.user} ${cfg.group} - -"

    # Jucenit file permissions
    "d '/var/spool/jucenit' 0774 ${cfg.user} users - -"
    "Z '/var/spool/jucenit' 0774 ${cfg.user} users - -"
    "d '/tmp/jucenit' 774 ${cfg.user} users - -"
    "Z '/tmp/jucenit' 774 ${cfg.user} users - -"
  ];

  ################################################
  ### Jucenit - autossl
  ## Systemd unit

  systemd.services.jucenit_autossl = {
    enable = true;
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    serviceConfig = {
      ExecStart = with pkgs; let
        jucenit = inputs.jucenit.packages.${system}.default;
      in ''
        ${jucenit} ssl --watch
      '';
      ReadWritePaths = [cfg.stateDir cfg.logDir cfg.challengDir];
    };
  };

  ################################################
  ### Nginx-unit
  ## Custom systemd unit
  # Replace default secure unix socket with local tcp socket
  # source at: https://github.com/NixOS/nixpkgs/nixos/modules/services/web-servers/unit/default.nix

  ## Add global packages
  # services.unit.enable = true; # Do not use and prefer custom unit

  systemd.services.unit = let
    settings = ''
      {
        "http": {
          "log_route": true,
        },
      }
    '';
  in {
    enable = true;
    after = ["network.target"];
    wantedBy = ["multi-user.target"];
    postStart = ''
      ${pkgs.curl}/bin/curl -X PUT --data-binary '${settings}' 'http://localhost:8080/config/settings'
    '';
    serviceConfig = {
      Type = "forking";
      PIDFile = "/run/unit/unit.pid";
      ExecStart = ''
        ${pkgs.unit}/bin/unitd \
          --control '127.0.0.1:8080' \
          --pid '/run/unit/unit.pid' \
          --log '${cfg.logDir}/unit.log' \
          --statedir '${cfg.stateDir}' \
          --tmpdir '/tmp' \
          --user unit \
          --group unit
      '';
      # Runtime directory and mode
      RuntimeDirectory = "unit";
      RuntimeDirectoryMode = "0750";
      # Access write directories
      ReadWritePaths = [cfg.stateDir cfg.logDir cfg.challengDir];
      # Security
      NoNewPrivileges = true;
      # Sandboxing
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      PrivateDevices = true;
      PrivateUsers = false;
      ProtectHostname = true;
      ProtectClock = true;
      ProtectKernelTunables = true;
      ProtectKernelModules = true;
      ProtectKernelLogs = true;
      ProtectControlGroups = true;
      RestrictAddressFamilies = ["AF_UNIX" "AF_INET" "AF_INET6"];
      LockPersonality = true;
      MemoryDenyWriteExecute = true;
      RestrictRealtime = true;
      RestrictSUIDSGID = true;
      PrivateMounts = true;
      # System Call Filtering
      SystemCallArchitectures = "native";
    };
  };
}
