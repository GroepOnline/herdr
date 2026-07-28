/**
 * Provider registry.
 *
 * Adding a source means writing one module with `{id, group, list, read}` and
 * appending it here. Everything else — sidebar grouping, tail mode, search,
 * export, headless actions — works off this list.
 */
import { GROUPS } from "../lib/channel.js";
import { agentsProvider } from "./agents.js";
import { customProvider } from "./custom.js";
import { fleetOpsProvider } from "./fleet-ops.js";
import { herdrLogsProvider } from "./herdr-logs.js";
import { mcpProvider } from "./mcp.js";
import { networkProvider } from "./network.js";
import { panesProvider } from "./panes.js";
import { pluginLogsProvider } from "./plugin-logs.js";
import { systemProvider } from "./system.js";

export const PROVIDERS = [
  panesProvider,
  herdrLogsProvider,
  pluginLogsProvider,
  fleetOpsProvider,
  mcpProvider,
  networkProvider,
  systemProvider,
  agentsProvider,
  customProvider,
];

const providerById = new Map(PROVIDERS.map((p) => [p.id, p]));

/**
 * List every channel from every provider, in group order.
 * A provider that throws degrades to a single error channel; it never takes
 * the whole panel down.
 */
export async function listChannels(ctx) {
  const settled = await Promise.all(
    PROVIDERS.map(async (provider) => {
      try {
        const channels = await provider.list(ctx);
        return channels.map((c) => ({ ...c, provider: provider.id }));
      } catch (err) {
        return [
          {
            id: `${provider.id}:__crashed`,
            title: `${provider.id} provider failed`,
            group: provider.group,
            subtitle: String(err?.message ?? err).slice(0, 100),
            source: provider.id,
            kind: "provider-error",
            provider: provider.id,
            meta: { error: String(err?.stack || err?.message || err) },
          },
        ];
      }
    }),
  );
  const all = settled.flat();
  const rank = (g) => {
    const i = GROUPS.indexOf(g);
    return i < 0 ? GROUPS.length : i;
  };
  return all.sort((a, b) => rank(a.group) - rank(b.group) || 0);
}

export function findChannel(channels, id) {
  return channels.find((c) => c.id === id) || null;
}

/** Read one channel. `channels` is the list from `listChannels`. */
export async function readChannel(channels, id, ctx, opts = {}) {
  const ch = findChannel(channels, id);
  if (!ch) {
    return {
      text: `unknown channel: ${id}\n\nhint: run list-channels to see valid ids`,
      ansi: false,
      source: "",
      note: "",
      error: "unknown channel",
      truncated: false,
    };
  }
  if (ch.kind === "provider-error") {
    return {
      text: ch.meta.error,
      ansi: false,
      source: ch.provider,
      note: "",
      error: "provider crashed",
      truncated: false,
    };
  }
  const provider = providerById.get(ch.provider);
  if (!provider) {
    return {
      text: `no provider registered for ${ch.provider}`,
      ansi: false,
      source: "",
      note: "",
      error: "no provider",
      truncated: false,
    };
  }
  try {
    return await provider.read(ch, ctx, opts);
  } catch (err) {
    return {
      text: `${provider.id}.read failed for ${id}\n\n${String(err?.stack || err?.message || err)}`,
      ansi: false,
      source: ch.source,
      note: "",
      error: String(err?.message ?? err),
      truncated: false,
    };
  }
}

export { GROUPS };
