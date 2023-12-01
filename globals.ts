// Types
import type { GetArgs, Options, SetArgs } from "./types.ts";

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
  }
  async set({ path, object }: SetArgs): Promise<void> {
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
  }
}
