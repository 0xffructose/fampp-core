# 🚀 FAMPP (Fast AMPP Stack)

![Rust](https://img.shields.io/badge/Built_with-Rust-f34b7d.svg?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg?style=flat-square)

![FAMPP Banner](Banner.png)

FAMPP is an ultra-fast, portable, and fully isolated local web development environment (Local Development Stack) engineered entirely from scratch in Rust. 

Designed for developers who are tired of clunky installation wizards, unwanted service file clutters, and cluttered interfaces. FAMPP runs everything within its own self-contained directory without installing any global dependencies on your host operating system.

## 🧠 The Philosophy

Traditional stacks like XAMPP or Docker can be overkill, resource-heavy, and prone to system conflicts. FAMPP takes a different approach:
* **Zero Global Pollution:** Everything lives in `~/.fampp`. If you want to uninstall, you simply delete the folder.
* **Zero External Dependencies:** No need for curl or bulky external formatting crates. FAMPP handles its own async network requests and draws its own pixel-perfect CLI UI natively.
* **Bare-Metal Performance:** Powered by a Rust CLI, ensuring negligible RAM consumption, instant CLI response times, and a footprint that makes heavy containers look obsolete.

## ✨ Key Features

- **🦀 Rust-Powered CLI Engine:** A robust, memory-safe daemon and process manager that controls your entire stack with zero overhead.
- **🌐 Built-in Asynchronous Downloader:** A custom network engine featuring smooth, colored progress bars (indicatif) showing real-time ETA and transfer speeds without relying on system curl.
- **⚙️ Global Configuration & Dynamic Ports:** Automatically generates a config.toml file, allowing you to easily customize ports (e.g., changing PHP from 8000 to 9000) and system behaviors.
- **🌍 Native i18n Localization Engine:** Fully supports multi-language CLI outputs (English and Turkish) directly controlled via the config file.
- **💅 Premium CLI UX:** Features custom-built Unicode status tables, dynamic loading animations, and a sleek ASCII art help menu—giving you a true "hacker" terminal aesthetic.
- **🐘 Portable PHP:** Runs a standalone, static PHP engine without interfering with your system's native binaries.
- **🛠️ Adminer Integration:** Ships with Adminer, a lightning-fast, single-file alternative to phpMyAdmin for instant database management.
- **🔍 Real-Time Log Tailer:** Stream background service logs (errors, access) directly to your terminal, just like a modern DevOps monitoring tool.

## 🎯 Perfect For

- Developing rapid local APIs for modern frontends (Next.js, React, Vue).
- Building robust, lightweight backends for tools or bots.
- Developers who build systems from scratch and demand total control over their environment.

## 🛠️ Installation & Setup

Since FAMPP is built with Rust, you can compile and run it directly using Cargo.

### 1. Clone & Build
```bash
git clone [https://github.com/0xffructose/fampp-core.git](https://github.com/0xffructose/fampp-core.git)
cd fampp
cargo build --release
```

### 2. Install the Stack
FAMPP will download and integrate the necessary binaries into its isolated environment.
```bash
cargo run -- install php
cargo run -- install mysql
cargo run -- install adminer
```
## 🚀 Usage Guide
FAMPP's CLI is designed to be intuitive and fast.
### Start your services:
```bash
cargo run -- start php
cargo run -- start mysql
```
*PHP will serve files from ~/.fampp/www at http://127.0.0.1:8000*
*MySQL will run locally on port 3306 with user root and no password.*
### Access the Database Manager:
Navigate to http://127.0.0.1:8000/adminer.php in your browser.

### Check System Status:
View running services, their PIDs, and uptime.
```bash
cargo run -- status
```
### Real-Time Log Monitoring:
Watch your background daemon logs live to catch errors instantly.
```bash
cargo run -- logs mysql
```

# 🗂️ Architecture & Directory Structure
FAMPP keeps your system clean by confining everything to a single hidden directory:
```bash
~/.fampp/
├── data/
│   └── mysql/         <-- Automatically generated MySQL database tables and system files
├── logs/              <-- Real-time log files for background services (e.g., mysql.log)
├── packages/
│   ├── php/           <-- Isolated, static PHP binary
│   └── mysql/         <-- Isolated MySQL engine
└── www/
    ├── adminer.php    <-- Single-file database manager
    └── /your_project  <-- Your application code
```

# 🗺️ Roadmap
[x] Core Rust Process Manager

[x] Custom Package Registry & Downloader

[x] PHP & MySQL Integration

[x] Real-time Log Tailer

[ ] Traffic Control: Nginx reverse proxy integration for custom .test local domains.
 
[ ] Localization files that are not embedded in the source code

[ ] Extended scope (PostgreSQL, MailHog etc.)

[ ] A well designed UI

<!--[ ] Full-Stack Expansion: Portable Node.js integration for JavaScript ecosystems.

[ ] Local Tunneling: Securely expose your local environment to the internet for webhook testing.-->

# 🤝 Contributing
FAMPP is an open-source initiative aimed at improving developer quality of life. Pull Requests, bug reports, and feature requests are highly welcome!

# 📄 License
This project is licensed under the MIT License - see the LICENSE file for details.