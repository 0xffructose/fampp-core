# 🚀 FAMPP (Fast AMPP Stack)

FAMPP is an ultra-fast, portable, and completely isolated local web development environment (Local Development Stack) written from scratch in Rust.

Say goodbye to clunky installation wizards, background services that pollute your system registry, or restrictive WAF/CDN download walls. FAMPP runs everything within its own directory without installing anything on your host operating system (macOS/Windows/Linux).

## ✨ Why FAMPP?

- **🦀 The Power of Rust Architecture:** Negligible RAM consumption, instant response times, and zero dependencies.
- **📦 Our Own Rules (Private Registry):** Bypasses the download walls of restrictive companies like Oracle by pulling packages directly from our own secure GitHub repository (CDN).
- **⚡ Isolated Environment:** Your projects, databases, and configurations live in a single, self-contained folder (`~/.fampp`). It never clutters your computer.
- **🔍 Real-time Log Tailer:** Just like modern DevOps tools, it allows you to monitor the logs of your background services instantly and live, right from your terminal.

## 🛠️ Installation & Usage

Using FAMPP is incredibly simple. Just run the following commands in your terminal:

```bash
# Integrate services into the system (Installation)
cargo run -- install php
cargo run -- install mysql
cargo run -- install adminer

# Start the services
cargo run -- start php
cargo run -- start mysql

# Check the status of the services (Uptime, PID numbers)
cargo run -- status

# Monitor error and access logs in real-time
cargo run -- logs mysql