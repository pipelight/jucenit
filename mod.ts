#!/usr/bin/env -S deno run -A

import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
import type { Options } from "./types.ts";
import { Unit } from "./globals.ts";
// import * from "actions.ts";
//
const cli = new Command()
  .name("nulite")
  .version("0.1.0")
  .description("Nginx-unit management cli");

// Connect to unit
cli
  .globalOption("--url", "nginx-unit url")
  .arguments("<value:string>")
  .globalOption("--socket", "nginx-unit socket")
  .arguments("<value:string>");

// Subcommands - Getters
cli
  .command("info", "Get every unit objects")
  .action((options: Options, ...args) => {
    const unit = new Unit(options);
    unit.get();
  })
  .command("config", "Get the unit configuration object")
  .action((options, ...args) => {
    const unit = new Unit(options);
    unit.get({ path: "/config" });
  })
  .command("certs", "Get the unit configuration object")
  .action((options, ...args) => {
    const unit = new Unit(options);
    unit.get({ path: "/certificates" });
  })
  .command("status", "Get the unit status object")
  .action((options, ...args) => {
    const unit = new Unit(options);
    unit.get({ path: "/status" });
  });

// Subcommands - Setters
cli
  .command("domain", "Set a new domain")
  .action((options, ...args) => {
    const unit = new Unit(options);
    unit.set({ path: "/config/listeners", object: {} });
    unit.set({ path: "/config/routes", object: {} });
  });

await cli.parse(Deno.args);
