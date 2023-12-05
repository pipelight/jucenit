import $ from "https://deno.land/x/dax/mod.ts";
import type { Container } from "https://deno.land/x/pipelight/mod.ts";
import { Unit } from "../class.ts";

export const actions = (unit: Unit) => {
  const make_routes = async (
    { globals, name, ports }: Partial<Container>,
  ): Promise<void> => {
    const port = ports!.shift();

    const data: any = {};
    data[`${name}`] = [
      {
        match: {
          host: globals!.dns,
        },
        action: {
          proxy: `http://${port!.ip}:${port!.out}`,
        },
      },
    ];
    await unit.set({ path: "/config/routes", object: data });
  };
  /**
  Create a listener
  */
  const make_listeners = async (
    { name }: Partial<Container>,
  ): Promise<void> => {
    const data = {
      // Listen on localhost
      // "127.0.0.1:80": {
      "*:80": {
        pass: `routes/${name}`,
      },
      // "127.0.0.1:443": {
      "*:443": {
        pass: `routes/${name}`,
        tls: {
          "certificate": `${name}`,
        },
      },
    };
    await unit.set({ path: "/config/listeners", object: data });
  };

  return {
    unit,
    make_routes,
    make_listeners,
  };
};
