# hoppr

hoppr is a simple CLI tool to download and update Minecraft mods or plugins from Modrinth using a CSV file.

Instead of dealing with JSON or TOML configs, you can manage your mod list in a spreadsheet or any text editor.

## Features

- **CSV Manifest:** Keep your mod list in a simple table.
- **Dependency Resolution:** Automatically adds required library dependencies.
- **Environment Filtering:** Filter downloads by environment (`--env client` or `--env server`).
- **Comment Support:** Comment out a row with `#` to temporarily disable a project.

## Quick Start

1. Initialize a manifest:
   ```bash
   hoppr init 1.21.1 fabric
   ```
2. Add mods:
   ```bash
   hoppr add sodium lithium
   ```
3. Download them:
   ```bash
   hoppr download
   ```

## Manifest Example (`manifest.csv`)

```csv
# version: 1
# minecraft: 1.21.1
# loader: fabric
id,version,environment
sodium,,c
# lithium,,s
voicechat,,
```

## Commands

- `hoppr init` — Create a new manifest.
- `hoppr add [ID/SLUG...]` — Add a project.
- `hoppr remove [ID]` — Remove a project.
- `hoppr download` — Download projects.
- `hoppr list` — Show current projects.
- `hoppr export-json` — Export to simple JSON.
