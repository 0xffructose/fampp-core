FAMPP (Fast AMPP Stack) is an ultra-fast, portable, and isolated local development environment written from scratch in Rust to ensure bare-metal performance. It eliminates the friction of traditional installation wizards and prevents system registry pollution by operating as a completely self-contained stack.

#### ✨ Key Features

* **The Power of Rust Architecture:**  Engineered for bare-metal performance with negligible RAM consumption, instant response times, and zero external dependencies.  
* **Private Registry (Our Own Rules):**  Bypasses restrictive WAF/CDN download walls—such as those imposed by Oracle—via a secure GitHub-based CDN for streamlined, pre-compiled asset delivery.  
* **Isolated Environment:**  Ensures the host system remains clutter-free by encapsulating all projects, databases, and configurations within the \~/.fampp directory.  
* **Real-time Log Tailer:**  Provides terminal-native monitoring for background services, offering instant, live visibility into the environment's internal state.

#### 🛠 Technical Architecture

The FAMPP core is developed 100% in Rust to provide maximum execution speed and seamless portability across macOS, Windows, and Linux. By decoupling the stack from the operating system's global configuration, FAMPP provides a predictable environment for local development.

##### Project Composition

Component,Description  
Language,Rust (100%)  
Environment Path,\~/.fampp (Self-contained)  
Release Assets,MySQL LTS Binaries

#### 🚀 Getting Started

FAMPP is designed for terminal-centric simplicity. You can deploy and manage the entire stack using the following commands:  
\# Download and install the FAMPP core  
curl \-fsSL https://raw.githubusercontent.com/0xffructose/fampp-core/main/install.sh | sh

\# Initialize and start the development services  
fampp start

#### 📂 Repository Overview

The codebase follows standard Rust conventions, organized for modularity and performance:

* **src/** : Application source code, featuring the core logic for CLI subcommand routing and service orchestration.  
* **Cargo.toml**  **&**  **Cargo.lock** : Manifests for Rust dependency management, defining the build lifecycle and ensuring reproducible environments.  
* **README.md** : Primary project documentation.

#### 📈 Project Status

This project is in active development and moving rapidly. Recent milestones include the initial release of pre-compiled MySQL LTS Binaries and a complete refactor of the internal binary routing logic. The current state reflects a high-velocity commit history focused on expanding core orchestration capabilities and refining system documentation.  
