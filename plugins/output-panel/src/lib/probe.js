/**
 * Read-only liveness probes for local services.
 *
 * TCP connect plus an optional HTTP GET. Everything is bounded by an explicit
 * timeout so a black-holed port cannot stall a channel read.
 */
import { createConnection } from "node:net";

import { redactText } from "./redact.js";

export function tcpProbe(host, port, timeoutMs = 1500) {
  return new Promise((resolve) => {
    const started = Date.now();
    const sock = createConnection({ host, port });
    let settled = false;
    const done = (open, detail) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      try {
        sock.destroy();
      } catch {
        // already closed
      }
      resolve({ open, ms: Date.now() - started, detail: detail || "" });
    };
    const timer = setTimeout(() => done(false, "timeout"), timeoutMs);
    timer.unref?.();
    sock.on("connect", () => done(true, ""));
    sock.on("error", (err) => done(false, err?.code || String(err?.message ?? err)));
  });
}

/** HTTP GET that never throws. Body is redacted and length-capped. */
export async function httpProbe(url, { timeoutMs = 2000, maxChars = 400 } = {}) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  timer.unref?.();
  const started = Date.now();
  try {
    const res = await fetch(url, {
      signal: controller.signal,
      redirect: "manual",
      headers: { accept: "*/*" },
    });
    let body = "";
    try {
      body = (await res.text()).slice(0, maxChars);
    } catch {
      body = "";
    }
    return {
      ok: true,
      status: res.status,
      ms: Date.now() - started,
      body: redactText(body.replace(/\s+/g, " ").trim()),
    };
  } catch (err) {
    const aborted = err?.name === "AbortError";
    return {
      ok: false,
      status: null,
      ms: Date.now() - started,
      error: aborted ? `timeout after ${timeoutMs}ms` : String(err?.cause?.code || err?.message || err),
    };
  } finally {
    clearTimeout(timer);
  }
}

/** Known local services worth probing. Kept in one place so both the */
/* Network and MCP providers describe them identically. */
export const LOCAL_SERVICES = [
  { name: "kater sse", host: "127.0.0.1", port: 9090, url: "http://127.0.0.1:9090/", mcp: true },
  { name: "kater rest", host: "127.0.0.1", port: 9091, url: "http://127.0.0.1:9091/health", mcp: true },
  { name: "brave cdp", host: "127.0.0.1", port: 9222, url: "http://127.0.0.1:9222/json/version", mcp: true },
  { name: "opencodex proxy", host: "127.0.0.1", port: 10100, url: "http://127.0.0.1:10100/" },
  { name: "joep-ops", host: "127.0.0.1", port: 10101, url: "http://127.0.0.1:10101/" },
  { name: "vault ui", host: "127.0.0.1", port: 8080, url: "http://127.0.0.1:8080/" },
  { name: "vault-api", host: "127.0.0.1", port: 8321, url: "http://127.0.0.1:8321/" },
  { name: "vaultwarden", host: "127.0.0.1", port: 8222, url: "http://127.0.0.1:8222/alive" },
  { name: "vault webtop", host: "127.0.0.1", port: 3000, url: "http://127.0.0.1:3000/" },
];

/** Probe one service and render it as a single aligned report line. */
export async function probeService(svc, timeoutMs = 1500) {
  const tcp = await tcpProbe(svc.host, svc.port, timeoutMs);
  const label = `${svc.name.padEnd(16)} ${svc.host}:${String(svc.port).padEnd(6)}`;
  if (!tcp.open) {
    return `${label} closed        (${tcp.detail || "no listener"})`;
  }
  if (!svc.url) return `${label} open   ${tcp.ms}ms`;
  const http = await httpProbe(svc.url, { timeoutMs });
  if (!http.ok) {
    return `${label} open   ${tcp.ms}ms  http: ${http.error}`;
  }
  const body = http.body ? `  ${http.body.slice(0, 120)}` : "";
  return `${label} open   ${tcp.ms}ms  HTTP ${http.status} ${http.ms}ms${body}`;
}
