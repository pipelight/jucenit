// Test
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Container, Docker } from "https://deno.land/x/pipelight/mod.ts";
// Self
import { Unit } from "./class.ts";
import { actions, certificate } from "./actions/mod.ts";

const docker = new Docker({
  globals: {
    dns: "example.com",
    version: "dev",
  },
  containers: [{
    suffix: "front",
    ports: [{
      in: 80,
      out: 8081,
    }],
  }],
});

Deno.test("expose container self-signed ssl", async () => {
  const container = docker.containers.get("front") as Container;
  console.debug(container);

  const unit = new Unit();
  const { dummy } = certificate(unit);
  await dummy(container);
});

Deno.test("expose container letsencrypt ssl", async () => {
  const container = docker.containers.get("front") as Container;
  console.debug(container);

  const unit = new Unit();
  const { dummy } = certificate(unit);
  await dummy(container);
});
