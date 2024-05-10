import type { Config } from "https://deno.land/x/pipelight/mod.ts";
import { pipeline, step } from "https://deno.land/x/pipelight/mod.ts";

const name = "dummy";

// Make a self signed cert
const openssl = pipeline("openssl", () => [
  step("ensure tmp dir", () => [
    "mkdir -p /tmp/jucenit",
  ]),
  step("generate cert", () => [
    `openssl req \
      -x509 -newkey rsa:4096 \
      -sha256 \
      -keyout /tmp/jucenit/key_${name}.pem \
      -out /tmp/jucenit/cert_${name}.pem \
      -days 3650 \
      -nodes \
      -subj '/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=example.com'`,
  ]),
]);

// Generate certs
const config = {
  pipelines: [openssl],
};

export default config;
