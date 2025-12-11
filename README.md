# Easy Remote

![Status](https://img.shields.io/badge/Status-Active-success)
![Language](https://img.shields.io/badge/Rust-2024-orange)
![GUI](https://img.shields.io/badge/GTK-4.0-blue)
![License](https://img.shields.io/badge/License-GPLv3-green)

**Easy Remote** is a simplified remote connection manager, developed in Rust with GTK4.

The main goal is to facilitate technical support in corporate Linux environments. Unlike traditional VNC viewers, Easy Remote focuses on **Reverse Connection**: the user clicks on the technician's name and the computer "sends" the screen to support.

## Features

- **Simple Interface:** Pre-configured list of technicians (Zero configuration for the end-user).
- **Reverse Connection:** Initiates connection from Client to Technician (Listening Mode).
- **Automatic Detection:**
  - **X11** support (via `x11vnc`).
  - Experimental **Wayland** support (via `wayvnc`).
- **Centralized Configuration:** Supports global (`/etc`) or per-user (`~/.config`) configuration files.
- **Visual Feedback:** Indicates connection status and errors directly in the interface without freezing the application.

## Installation

### Ubuntu / Debian / Mint

Download the latest `.deb` package from the [Releases](https://github.com/RuanVasco/easy-remote/releases) tab and install:

```bash
sudo apt install ./easy-remote_0.1.0_amd64.deb
```
