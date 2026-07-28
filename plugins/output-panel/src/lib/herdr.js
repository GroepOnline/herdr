/**
 * Herdr access: the CLI for reads, the Unix socket for actions.
 */
import { createConnection } from "node:net";

import { run } from "./exec.js";
import { herdrBin, herdrSocket } from "./paths.js";

/** Run a `herdr` subcommand and return the raw exec result. */
export function herdrRun(args, options = {}) {
  return run(herdrBin(), args, options);
}

/** Run a `herdr` subcommand that emits JSON and return `result`, or null. */
export async function herdrJson(args, options = {}) {
  const res = await herdrRun(args, options);
  if (!res.ok) return { result: null, res };
  try {
    const parsed = JSON.parse(res.stdout);
    return { result: parsed?.result ?? parsed, res };
  } catch {
    return { result: null, res };
  }
}

/**
 * Newline-delimited JSON request against the Herdr API socket.
 * Used for side effects (pane.focus) the CLI does not expose directly.
 */
export function herdrRpc(method, params = {}, timeoutMs = 3000) {
  return new Promise((resolve, reject) => {
    const payload = `${JSON.stringify({
      id: `output-panel:${Date.now()}`,
      method,
      params,
    })}\n`;
    const sock = createConnection(herdrSocket());
    let buf = "";
    const timer = setTimeout(() => {
      sock.destroy();
      reject(new Error(`herdr rpc timeout (${method})`));
    }, timeoutMs);
    timer.unref?.();
    const done = (fn, arg) => {
      clearTimeout(timer);
      fn(arg);
    };
    sock.on("connect", () => sock.write(payload));
    sock.on("data", (chunk) => {
      buf += chunk.toString("utf8");
      const nl = buf.indexOf("\n");
      if (nl < 0) return;
      sock.end();
      try {
        done(resolve, JSON.parse(buf.slice(0, nl)));
      } catch (err) {
        done(reject, err);
      }
    });
    sock.on("error", (err) => done(reject, err));
  });
}
