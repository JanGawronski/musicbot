{
  description = "Music bot service";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      pkgs = import nixpkgs { system = "x86_64-linux"; };

      pkg = pkgs.rustPlatform.buildRustPackage {
        pname = "musicbot";
	      version = "0.9.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;	
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs; [ openssl libopus ];
      };
    
      module = { config, lib, pkgs, ... }: {
        options.services.musicbot.enable = lib.mkOption {
          type = lib.types.bool;
          default = false;
          description = "Enable the musicbot systemd service.";
        };

        options.services.musicbot.audioDirectory = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = null;
          description = "Directory for local audio files to be used by bot";
        };

        options.services.musicbot.cookiesPath = lib.mkOption {
          type = lib.types.nullOr lib.types.str;
          default = null;
          description = "Path to cookies file to be used by bot";
        };

        options.services.musicbot.user = lib.mkOption {
          type = lib.types.str;
          default = "musicbot";
          description = "User to run systemd service";
        };

        options.services.musicbot.group = lib.mkOption {
          type = lib.types.str;
          default = "musicbot";
          description = "Group of user to run systemd service";
        };

        options.services.musicbot.discordTokenPath = lib.mkOption {
          type = lib.types.str;
          description = "Path to token for discord bot";
        };

        config = lib.mkIf config.services.musicbot.enable {
          systemd.services.musicbot = {
            description = "Music bot service";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];
            serviceConfig = {
              ExecStart = "${self.packages.musicbot}/bin/musicbot --yt-dlp=${pkgs.yt-dlp}/bin/yt-dlp --token=${config.services.musicbot.discordTokenPath} ${lib.optionalString (config.services.musicbot.audioDirectory != null) "--local-audio=${config.services.musicbot.audioDirectory}"} ${lib.optionalString (config.services.musicbot.cookiesPath != null) "--cookies=${config.services.musicbot.cookiesPath}"}";
              Restart = "on-failure";
              RestartSec = "5s";
              StandardOutput = "journal";
              StandardError = "journal";
            };
            serviceConfig.User = config.services.musicbot.user;
            serviceConfig.Group = config.services.musicbot.group;
          };
          
          users.groups = lib.optionalAttrs config.services.musicbot.enable {
            "${config.services.musicbot.group}" = { };
          };

          users.users = lib.optionalAttrs config.services.musicbot.enable {
            "${config.services.musicbot.user}" = {
              isSystemUser = true;
              group = "musicbot";
              description = "User for musicbot service";
              createHome = false;
              home = "/var/lib/musicbot";
            };
          };
        };
      };
    in {
      packages.musicbot = pkg;
      nixosModule = module;
    };
}
