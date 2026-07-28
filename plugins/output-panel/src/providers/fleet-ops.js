/**
 * Plugin state files under ~/.local/state/herdr/plugins/<plugin>/*.json.
 *
 * `fleet_ops.json` is the Fleet Ops Bar fragment; sibling JSON files (kater
 * doctor/PR snapshots and friends) are exposed too. Everything is
 * pretty-printed through the redactor.
 */
import path from "node:path";

import { channel, failure, result } from "../lib/channel.js";
import { humanAge, listDir, readJson, statOrNull } from "../lib/files.js";
import { pluginStateRoot, tilde } from "../lib/paths.js";
import { redactedJsonString } from "../lib/redact.js";

const PREFIX = "state:";

export const fleetOpsProvider = {
  id: "fleet-ops",
  group: "Plugins",

  async list() {
    const root = pluginStateRoot();
    const entries = await listDir(root);
    const channels = [];
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      const dir = path.join(root, entry.name);
      const files = await listDir(dir);
      for (const file of files) {
        if (!file.isFile() || !file.name.endsWith(".json")) continue;
        const full = path.join(dir, file.name);
        const st = await statOrNull(full);
        if (!st) continue;
        const short = entry.name.replace(/^com\.chefgroep\./, "");
        channels.push(
          channel({
            id: `${PREFIX}${entry.name}:${file.name}`,
            title: `${short} · ${file.name}`,
            group: "Plugins",
            subtitle: `${humanAge(st.mtimeMs)}  ${tilde(full)}`,
            source: full,
            kind: "json",
            meta: { path: full, plugin: entry.name },
          }),
        );
      }
    }
    if (!channels.length) {
      channels.push(
        channel({
          id: `${PREFIX}__empty`,
          title: "no plugin state files",
          group: "Plugins",
          subtitle: tilde(root),
          source: root,
          kind: "error",
          meta: { dir: root },
        }),
      );
    }
    return channels.sort((a, b) => a.id.localeCompare(b.id));
  },

  async read(ch) {
    if (ch.kind === "error") {
      return failure({
        tried: ch.meta.dir,
        reason: "no plugin has written state yet",
        hint: "plugins write fleet_ops.json on their first event; trigger one and press R",
      });
    }
    try {
      const data = await readJson(ch.meta.path);
      return result(redactedJsonString(data), {
        raw: true,
        source: ch.meta.path,
        note: "values matching token/key/password patterns are masked",
      });
    } catch (err) {
      return failure({
        tried: ch.meta.path,
        reason: String(err?.message ?? err),
        hint: "the plugin may be mid-write; press R to retry",
      });
    }
  },
};
