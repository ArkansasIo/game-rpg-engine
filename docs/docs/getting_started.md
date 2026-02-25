---
title: "Getting Started"
sidebar_position: 1
---

![Logo](/img/eldiron-banner.png)

## Install Options

### Prebuilt Releases

Download binaries from GitHub Releases:

- <https://github.com/markusmoenig/Eldiron/releases>

### Build From Source

```bash
git clone https://github.com/markusmoenig/Eldiron
cd Eldiron
cargo run -p eldiron-creator
```

Release mode:

```bash
cargo run --release -p eldiron-creator
```

### Linux Dependencies

Install:

- `libasound2-dev`
- `libatk1.0-dev`
- `libgtk-3-dev`

## First Project Flow

1. Create or open a project in Eldiron Creator.
2. Build a map with the tile and geometry tools.
3. Add characters, items, and events.
4. Configure game behavior in project settings.
5. Test in the client runtime.

## Recommended Learning Order

1. [Working with Geometry](/docs/working_with_geometry)
2. [Building Maps](/docs/building_maps/working_with_tiles)
3. [2D or 3D Maps?](/docs/building_maps/2d_or_3d)
4. [Characters & Items](/docs/characters_items/getting_started)
5. [Clients](/docs/clients)
