// Types
import type { GetArgs, Options, SetArgs } from "./types.ts";
import { UnitError } from "./types.ts";
import type { Container } from "https://deno.land/x/pipelight/mod.ts";
import $ from "https://deno.land/x/dax/mod.ts";

// Globals
export class Unit {
  url: string = "http://127.0.0.1:8080";
  socket?: string;
  constructor(params?: Options) {
    if (params){

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
  async make_routes(
    { globals, name, ports }: Partial<Container>,
  ): Promise<void> {
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
    await this.set({ path: "/config/routes", object: data });
  }
  /**
  Create a self-signed cerificate
  */
  async make_dummy_cert({ globals, name }: Partial<Container>): Promise<void> {
    const tmp_dir = ".pipelight/tmp";
    // Create temp dir
    await $`mkdir -p ${tmp_dir}`;
    // Generate certs
    await $`openssl req \
        -x509 -newkey rsa:4096 \
        -sha256 \
        -keyout ${tmp_dir}/key_${name}.pem \
        -out ${tmp_dir}/cert_${name}.pem \
        -days 3650 \
        -nodes \
        -subj '/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=example.com'`;

    // Concat certs into bundle
    //
    // Deprecated
    // await $`cat ${tmp_dir}/cert_${name}.pem ${tmp_dir}/key_${name}.pem > ${tmp_dir}/bundle_${name}.pem`;
    //
    const cert = await Deno.readTextFile(`${tmp_dir}/cert_${name}.pem`);
    const key = await Deno.readTextFile(`${tmp_dir}/key_${name}.pem`);
    await Deno.writeTextFile(`${tmp_dir}/bundle_${name}.pem`, cert + key);
    const bundle = await Deno.readTextFile(`${tmp_dir}/bundle_${name}.pem`);
    // console.trace(bundle);
    try {
      await this.get({
        path: `/certificates/${name}`,
      });
    } catch {
    }

    await this.set({
      path: `/certificates/${name}`,
      raw: bundle,
    });

    // Clean up
    // Remove files
    await $`rm ${tmp_dir}/cert_${name}.pem ${tmp_dir}/key_${name}.pem ${tmp_dir}/bundle_${name}.pem`;
  }
  /**
  Create an official cerificate
  */
  async make_cert(
    { globals, name, ports }: Partial<Container>,
  ): Promise<void> {
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
    await this.set({ path: "/config/certificates", object: data });
  }
  /**
  Create a listener
  */
  async make_listeners({ name }: Partial<Container>): Promise<void> {
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
    await this.set({ path: "/config/listeners", object: data });
  }
}
