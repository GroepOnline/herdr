/**
 * Async process execution with a hard timeout.
 *
 * Every channel provider goes through here so a slow command (journalctl,
 * a network probe, a wedged CLI) can never block the UI thread or hang a
 * headless action.
 */
import { spawn } from "node:child_process";

export const DEFAULT_TIMEOUT_MS = 3000;

/**
 * Run a command and always resolve, never reject.
 *
 * @returns {Promise<{ok: boolean, code: number|null, stdout: string, stderr: string,
 *                    timedOut: boolean, missing: boolean, argv: string}>}
 */
export function run(cmd, args = [], options = {}) {
  const {
    timeoutMs = DEFAULT_TIMEOUT_MS,
    env = process.env,
    cwd = undefined,
    maxBytes = 4 * 1024 * 1024,
  } = options;
  const argv = [cmd, ...args].join(" ");

  return new Promise((resolve) => {
    let child;
    try {
      child = spawn(cmd, args, { stdio: ["ignore", "pipe", "pipe"], env, cwd });
    } catch (err) {
      resolve({
        ok: false,
        code: null,
        stdout: "",
        stderr: String(err?.message ?? err),
        timedOut: false,
        missing: true,
        argv,
      });
      return;
    }

    let stdout = "";
    let stderr = "";
    let settled = false;
    let timedOut = false;
    let missing = false;

    const finish = (code) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      resolve({
        ok: code === 0 && !timedOut,
        code,
        stdout,
        stderr,
        timedOut,
        missing,
        argv,
      });
    };

    const timer = setTimeout(() => {
      timedOut = true;
      try {
        child.kill("SIGKILL");
      } catch {
        // process already gone
      }
      finish(null);
    }, timeoutMs);
    timer.unref?.();

    child.stdout.on("data", (chunk) => {
      if (stdout.length < maxBytes) stdout += chunk.toString("utf8");
    });
    child.stderr.on("data", (chunk) => {
      if (stderr.length < maxBytes) stderr += chunk.toString("utf8");
    });
    child.on("error", (err) => {
      missing = err?.code === "ENOENT";
      stderr += String(err?.message ?? err);
      finish(null);
    });
    child.on("close", (code) => finish(code));
  });
}

/** Run and return stdout, or null when the command failed. */
export async function runStdout(cmd, args, options) {
  const res = await run(cmd, args, options);
  return res.ok ? res.stdout : null;
}

/** Turn a failed `run()` result into a line that says what to do about it. */
export function explainFailure(res, hint = "") {
  const parts = [];
  if (res.missing) parts.push(`command not found: ${res.argv.split(" ")[0]}`);
  else if (res.timedOut) parts.push(`timed out: ${res.argv}`);
  else parts.push(`exit ${res.code ?? "?"}: ${res.argv}`);
  const err = (res.stderr || "").trim().split(/\r?\n/).slice(0, 4).join("\n");
  if (err) parts.push(err);
  if (hint) parts.push(hint);
  return parts.join("\n");
}
