---
title: "Clients"
sidebar_position: 7
---

# Clients

In [Getting Started](getting_started), we discussed how to download the binary files for **NexusStudio Creator**.

Here we cover the **NexusStudio Clients**, i.e. stand-alone apps that can play the game you created.

In the **Assets** folder of the latest GitHub release [here](https://github.com/markusmoenig/NexusStudio/releases), you can download the following files.

For **Windows**, download

* **nexusstudio-client-x86_64-pc-windows-msvc.zip**

For **Linux**, download

* **nexusstudio-client-x86_64-unknown-linux-gnu.tar.gz**

For the **Web / WASM**, download

* **nexusstudio-client-wasm32-unknown-unknown.tar.gz**

For **macOS**, we currently do not have a pre-built binary.

### Install via Cargo

If you have [Rust installed](https://www.rust-lang.org/tools/install), you can install NexusStudio Creator directly from [crates.io](https://crates.io):

```bash
cargo install nexusstudio-client
```

# Running your Game

On **Linux** and **Windows**, just start the client and pass the name of the **.nexusstudio** game file on the command line.

For the **Web**, rename your **.nexusstudio** file to **game.nexusstudio** and put it in the same directory as **index.html** and the other files. You can then run the game on any web server.

# Binary Files

Right now the **client** works directly on **.nexusstudio** source files. As we move closer to a v1 of **NexusStudio**, I will add the export of **binary** project files from within **NexusStudio Creator**. These binary files cannot be loaded back into the **Creator** and can only be run by the clients.
