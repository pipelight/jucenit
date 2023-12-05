#!/usr/bin/env -S deno run -A

import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
import { Container, Docker } from "https://deno.land/x/pipelight/mod.ts";
import type { Options } from "./types.ts";
import { Unit } from "./class.ts";
import { actions, certificate } from "./actions/mod.ts";
//
const cli = new Command()
  .name("jucenit")
  .version("0.1.0")
  .description("Nginx-unit management cli");

// Connect to unit
cli
  .globalOption("--url", "nginx-unit url")
  .arguments("<value:string>")
  .globalOption("--socket", "nginx-unit socket")
  .arguments("<value:string>")
  .globalOption("--log", "Set the loglevel")
  .arguments("<value:string>");

// Subcommands - Getters
cli
  .command("info", "Get every unit objects")
  .action((options: Options) => {
    const unit = new Unit(options);
    unit.get({ path: "/" });
  })
  .command("config", "Get the unit configuration object")
  .action((options: Options) => {
    const unit = new Unit(options);
    unit.get({ path: "/config" });
  })
  .command("certs", "Get the unit configuration object")
  .action((options: Options) => {
    const unit = new Unit(options);
    unit.get({ path: "/certificates" });
  })
  .command("status", "Get the unit status object")
  .action((options: Options) => {
    const unit = new Unit(options);
    unit.get({ path: "/status" });
  });

// Subcommands - Setters
cli
  .command("domain", "Set a new domain")
  .option("--dummy", "Generate a self signed dummy socket")
  .option("--dry-run", "Certbot dry-run")
  .action((options, ...args) => {
    const unit = new Unit(options as Options);

    const container = args.shift();

    if (options.dummy) {
      const { dummy } = certificate(unit);
      dummy(container);
    }

    // unit.make_routes(container);
    // unit.make_listeners(container);
  });

await cli.parse(Deno.args);
