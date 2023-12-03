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
};

export class UnitError extends Error {
  constructor(json: any) {
    super(json.error);
    this.name = "NginxUnitError";
  }
}
