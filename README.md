# hoppr

hoppr is a simple CLI tool to download and update Minecraft mods or plugins from Modrinth using a CSV file.

Instead of dealing with JSON or TOML configs, you can manage your mod list in a spreadsheet or any text editor.

## Features

- **CSV Manifest:** Keep your mod list in a simple table.
- **Dependency Resolution:** Automatically adds required library dependencies.
- **Environment Filtering:** Filter downloads by environment (`--env client` or `--env server`).
- **Comment Support:** Comment out a row with `#` to temporarily disable a project.
- - **Smart Updates:** Update your entire manifest or specific projects, with the ability to pin versions to prevent changes.

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
sodium,=mc1.21-0.5.11,c     # Pinned version (won't be updated)
lithium,,s                 # Tracks the latest version
voicechat,2.6.18,          # Specific version (will be updated on 'hoppr update')
```

## Commands

- `hoppr init` — Create a new manifest.
- `hoppr add [ID/SLUG...]` — Add a project.
- `hoppr remove [ID]` — Remove a project.
- `hoppr update [ID...]` — Update the entire manifest or specific projects to the latest compatible versions.
- `hoppr update -c [ID...]` — Check for available updates without applying them.
- `hoppr download` — Download projects.
- `hoppr list` — Show current projects.
- `hoppr export-json` — Export to simple JSON.

Use `hoppr --help` or `hoppr [command] --help` for more details on available options and flags.
