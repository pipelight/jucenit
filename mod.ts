#!/usr/bin/env -S deno run -A

import { Command } from "https://deno.land/x/cliffy/command/mod.ts";
// import * from "actions.ts";
//
type Options = {
  url?: string | Url;
  socket?: string | socket;
};

const unit = {
  url: "http://127.0.0.1:8080",
};

let init = (options): void => {
  if (options?.url) {
    url = options.url;
  }
};

let get = async (): void => {
  const request = new Request(unit.url, {
    method: "GET",
    // headers: {
    //   "content-type": "application/json",
    // },
  });
  const res = await fetch(request);
  const json = await res.json();
  console.log(json);
};

const cli = new Command()
  .name("nulite")
  .version("0.1.0")
  .description("Nginx-unit management cli");

// Connect to unit
cli
  .globalOption("--url", "nginx-unit url")
  .globalOption("--socket", "nginx-unit socket");

// Subcommands
cli
  .command("foo", "Foo sub-command.")
  .action((options, ...args) => {
    init(options);
    get();
  });

await cli.parse(Deno.args);

const update = () => {
  const request = new Request(url, {
    method: "POST",
    body: JSON.stringify({
      message: "Hello world!",
    }),
    headers: {
      "content-type": "application/json",
    },
  });
};
