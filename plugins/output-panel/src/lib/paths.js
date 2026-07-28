/**
 * Path resolution and the shared provider context.
 *
 * Herdr injects HERDR_PLUGIN_STATE_DIR / HERDR_PLUGIN_CONFIG_DIR when it runs
 * a plugin command. When the plugin is run by hand those fall back to the
 * documented locations under ~/.config and ~/.local/state.
 */
import { homedir } from "node:os";
import path from "node:path";

export const PLUGIN_ID = "com.chefgroep.output-panel";

export const HOME = homedir();

export const herdrBin = () => process.env.HERDR_BIN_PATH || "herdr";

export const herdrConfigDir = () =>
  process.env.HERDR_CONFIG_DIR || path.join(HOME, ".config/herdr");

export const herdrSocket = () =>
  process.env.HERDR_SOCKET_PATH ||
  process.env.HERDR_SOCKET ||
  path.join(herdrConfigDir(), "herdr.sock");

export const pluginStateRoot = () => path.join(HOME, ".local/state/herdr/plugins");

export const pluginStateDir = () =>
  process.env.HERDR_PLUGIN_STATE_DIR || path.join(pluginStateRoot(), PLUGIN_ID);

export const pluginConfigDir = () =>
  process.env.HERDR_PLUGIN_CONFIG_DIR ||
  path.join(herdrConfigDir(), "plugins/config", PLUGIN_ID);

export const cursorLogsRoot = () => path.join(HOME, ".config/Cursor/logs");
export const cursorHome = () => path.join(HOME, ".cursor");
export const cursorMcpConfig = () => path.join(cursorHome(), "mcp.json");
export const piSessionsRoot = () => path.join(HOME, ".pi/agent/sessions");
export const cursorProjectsRoot = () => path.join(cursorHome(), "projects");

/** Replace $HOME with ~ so paths stay readable in narrow panes. */
export function tilde(p) {
  if (!p) return p;
  return p.startsWith(HOME) ? `~${p.slice(HOME.length)}` : p;
}

/** Build the context handed to every provider. */
export function makeContext(overrides = {}) {
  return {
    lines: Number(process.env.OUTPUT_PANEL_LINES || 400),
    timeoutMs: Number(process.env.OUTPUT_PANEL_TIMEOUT_MS || 3000),
    herdrBin: herdrBin(),
    home: HOME,
    configDir: pluginConfigDir(),
    stateDir: pluginStateDir(),
    ...overrides,
  };
}
