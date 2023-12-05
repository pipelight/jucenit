// Types
// Self
import type { GetArgs, Options, SetArgs } from "./types.ts";
import { UnitError } from "./types.ts";

/**
This class contains the methods to query the unitd socket:

 - get (send a GET request with fetch)
 - set (send a PUT request with fetch)
 - update (send a POST request with fetch)
 - delete (send a DELETE request with fetch)

They are used as a base block for more complex actions.

*/
export class Unit {
  url: string = "http://127.0.0.1:8080";
  socket?: string;
  constructor(params?: Options) {
    if (params) {
      this.url ?? params.url;
      this.socket ?? params.socket;
    }
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
    // console.debug(request);

    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async update({ path, object, raw }: SetArgs): Promise<void> {
    const url = path ? this.url + path : this.url;

    const request = new Request(url, {
      method: "POST",
      body: object
        ? JSON.stringify(
          object,
        )
        : raw,
      headers: {
        "content-type": "application/json",
      },
    });
    // console.debug(request);

    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async set({ path, object, raw }: SetArgs): Promise<void> {
    const url = path ? this.url + path : this.url;

    const request = new Request(url, {
      method: "PUT",
      body: object
        ? JSON.stringify(
          object,
        )
        : raw,
      headers: {
        "content-type": "application/json",
      },
    });
    // console.debug(request);

    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
  async delete(args?: GetArgs): Promise<void> {
    const { path } = args!;

    const url = path ? this.url + path : this.url;
    const request = new Request(url, {
      method: "DELETE",
      headers: {
        "content-type": "application/json",
      },
    });
    console.debug(request);

    const res = await fetch(request);
    const json = await res.json();
    console.log(json);

    if (json.error) {
      throw new UnitError(json);
    }
  }
}
