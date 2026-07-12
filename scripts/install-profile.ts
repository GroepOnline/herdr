// install-profile — install OnlineChefGroep herdr profile
import { $ } from "bun";

const configPath = `${Bun.env.HOME}/.config/herdr/config.toml`;

const chefDefaults = `
# OnlineChefGroep herdr profile
# Applied by install-profile.ts — does not overwrite existing config

[keys]
prefix = "ctrl+a"

[ui]
redraw_on_focus_gained = false
host_cursor = "auto"

[theme]
name = "catppuccin"

[terminal]
default_shell = "/usr/bin/fish"
`;

console.log("Installing OnlineChefGroep herdr profile...");

const exists = Bun.file(configPath).exists();
if (exists) {
  const backup = `${configPath}.backup-chef-${Date.now()}`;
  await Bun.write(backup, Bun.file(configPath));
  console.log(`Backed up to ${backup}`);
}

await Bun.write(configPath, chefDefaults);
console.log(`Profile installed to ${configPath}`);
console.log("Run 'herdr server reload-config' to apply.");