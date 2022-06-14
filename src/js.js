// Node detection taken from https://www.npmjs.com/package/browser-or-node
export const IS_NODE =
  typeof process !== "undefined" &&
  process.versions != null &&
  process.versions.node != null;

// TODO 
export function web_crypto() {
  return self.crypto || self.msCrypto;
}

// TODO
export var NODE_CRYPTO = await import("node:crypto").catch(_ => { })
