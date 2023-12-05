import $ from "https://deno.land/x/dax/mod.ts";
import type { Container } from "https://deno.land/x/pipelight/mod.ts";
import { Unit } from "../class.ts";

export const certificate = (unit: Unit) => {
  /**
  Upload certficate to unit
  And try to delete previously set certificate
  */
  type UploadArgs = {
    name?: string;
    bundle?: string;
  };
  const upload = async ({
    name,
    bundle,
  }: UploadArgs): Promise<void> => {
    // Delete previous certificate
    try {
      await unit.delete({
        path: `/certificates/${name}`,
      });
    } catch (_err) {}

    await unit.set({
      path: `/certificates/${name}`,
      raw: bundle,
    });
  };
  /**
  Create a self-signed cerificate
  */
  const openssl = async (
    { name }: Partial<Container>,
  ): Promise<void> => {
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
    await upload({ name, bundle });

    // Clean up
    // Remove tmp files
    await $`rm ${tmp_dir}/cert_${name}.pem ${tmp_dir}/key_${name}.pem ${tmp_dir}/bundle_${name}.pem`;
  };

  /**
  Create an official cerificate with local certbot/letsencrypt
  */
  const letsencrypt = async (
    { globals, name }: Partial<Container>,
  ): Promise<void> => {
    const tmp_dir = ".pipelight/tmp/letsencrypt";
    // Create temp dir
    await $`mkdir -p ${tmp_dir}`;

    // Generate certs
    await $`certbot certonly \
      -d ${globals!.dns} \
      --manual \
      --dry-run \
      --email test@example.com \
      --work-dir ${tmp_dir} \
      --config-dir ${tmp_dir}/config \
      --logs-dir ${tmp_dir}/logs`;

    const bundle = await Deno.readTextFile(`${tmp_dir}/live/${name}/chain.pem`);
    await upload({ name, bundle });
  };

  return {
    dummy: openssl,
    pro: letsencrypt,
  };
};
