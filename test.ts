// Test
import { assertEquals } from "https://deno.land/std/assert/mod.ts";
import { Container, Docker } from "https://deno.land/x/pipelight/mod.ts";
// Self
import { Unit } from "./class.ts";

Deno.test("expose container self-signed ssl", () => {
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
  const container = docker.containers.get("front") as Container;

  const unit = new Unit();
  unit.make_dummy_cert(container);
});
