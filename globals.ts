// Types
import type { GetArgs, Options, SetArgs } from "./types.ts";
import { UnitError } from "./types.ts";
import type { Container } from "https://deno.land/x/pipelight/mod.ts";

// Globals
export class Unit {
  url: string = "http://127.0.0.1:8080";
  socket?: string;
  constructor(params: Options) {
    this.url ?? params.url;
    this.socket ?? params.socket;
  }
  // Methods
  async get(args?: GetArgs): Promise<void> {
    const { path } = args!;

    const url = path ? this.url + path : this.url;
    const request = new Request(url, {
      method: "GET",
      headers: {
        "content-type": "application/json",
      },
    });

    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async update({ path, object }: SetArgs): Promise<void> {
    const url = path ? this.url + path : this.url;

    const request = new Request(url, {
      method: "POST",
      body: JSON.stringify(
        object,
      ),
      headers: {
        "content-type": "application/json",
      },
    });
    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async set({ path, object }: SetArgs): Promise<void> {
    const url = path ? this.url + path : this.url;

    const request = new Request(url, {
      method: "PUT",
      body: JSON.stringify(
        object,
      ),
      headers: {
        "content-type": "application/json",
      },
    });
    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async make_routes({ globals, ports }: Partial<Container>): Promise<void> {
    const port = ports!.shift();
    const name = `${globals!.version}.${globals!.dns}`;

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
    await this.set({ path: "/config/routes", object: data });
  }
  async make_listeners({ globals, ports }: Partial<Container>): Promise<void> {
    const name = `${globals!.version}.${globals!.dns}`;

    const data = {
      // Listen on localhost
      "127.0.0.1:80": {
        pass: `routes/${name}`,
      },
      "127.0.0.1:443": {
        pass: `routes/${name}`,
        tls: {
          "certificate": `${name}`,
        },
      },
    };
    await this.set({ path: "/config/listeners", object: data });
  }
}
