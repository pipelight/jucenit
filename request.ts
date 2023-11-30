
const make_unit = (url?: string) => {
  // Nginx-unit server url
  url = url ?? unit_default_url;

  const config = {
    init: () => {
      const object = {
        listeners: {},
        routes: {},
      };
      const data = JSON.stringify(object);
      const req = `curl -X PUT \  
    --data-binary '${data}' \ 
    ${url}/config`;
      return req;
    },
  };

  const routes = {
    insert: (object: any) => {
      const data = JSON.stringify(object);
      const req = `curl -X PUT \
    --data-binary '${data}' \
    ${url}/config/routes`;
      return req;
    },
  };

  const listeners = {
    delete: (object: any) => {
      const data = JSON.stringify(object);
      const req = `curl -X DELETE \
    --data-binary '${data}' \
    ${url}/config/listeners`;
      return req;
    },
    insert: (object: any) => {
      const data = JSON.stringify(object);
      const req = `curl -X PUT \
    --data-binary '${data}' \
    ${url}/config/listeners`;
      return req;
    },
  };

  const tmp_dir = ".pipelight/tmp";

  const certificate = {
    delete: ({ name, id }: Container) => {
      // Remove previous cert
      return `curl -X DELETE \
    ${url}/certificates/${name}`;
    },
    insert: ({ name, id }: Container) => {
      return `curl -X PUT \
    --data-binary @${tmp_dir}/bundle_${name}.pem \
    ${url}/certificates/${name}`;
    },
  };

  /**
Create an nginx-unit route.
  */
  const make_routes = ({ globals, name, ports }: Container): string[] => {
    const port = ports![0];
    const data: any = {};
    data[`${name}`] = [
      {
        match: {
          host: globals!.dns,
        },
        action: {
          proxy: `http://${port.ip}:${port.out}`,
        },
      },
    ];
    let req = update_routes(data);
    return [req];
  };

  /**
Create an nginx-unit listener.
  */
  const make_listeners = ({ name }: Container): string[] => {
    let data = {
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
    let req = update_listeners(data);
    return [req];
  };
  /**
Create an nginx-unit listener.
  */
  const make_certificates = (container: Container): string[] => {
    const name = container.name;
    const tmp_dir = ".pipelight/tmp";
    // Generate a dummy certificate
    const dummy_cert = `openssl req \
        -x509 -newkey rsa:4096 \
        -sha256 \
        -keyout ${tmp_dir}/key_${name}.pem \
        -out ${tmp_dir}/cert_${name}.pem \
        -days 3650 \
        -nodes \
        -subj '/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=example.com'`;

    // const bundle = `cat cert_${name}.pem ca.pem key_${name}.pem > bundle_${name}.pem`;
    const make_tmp_dir = "mkdir -p .pipelight/tmp";
    const bundle =
      `cat ${tmp_dir}/cert_${name}.pem ${tmp_dir}/key_${name}.pem > ${tmp_dir}/bundle_${name}.pem`;
    const remove_tmp_files =
      `rm ${tmp_dir}/cert_${name}.pem ${tmp_dir}/key_${name}.pem ${tmp_dir}/bundle_${name}.pem`;

    return [
      make_tmp_dir,
      dummy_cert,
      bundle,
      ...update_certificates(container),
      remove_tmp_files,
    ];
  };

  const expose = (container: Container) => {
    return [
      ...make_routes(container),
      ...make_listeners(container),
      ...make_certificates(container),
    ];
  };

  return {
    init_config,
    expose,
    make_listeners,
    make_routes,
    make_certificates,
  };
};

export { make_unit, make_unit as nginx_unit };
