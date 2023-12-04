// Types
export type Options = {
  url?: string | URL;
  socket?: string;
};

export type GetArgs = {
  path?: string;
};

export type SetArgs = {
  path?: string;
  object?: any;
  raw?: string;
};

export class UnitError extends Error {
  constructor(json: any) {
    super(json.error);
    this.name = "NginxUnitError";
  }
}
