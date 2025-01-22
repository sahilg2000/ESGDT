# ESGDT
Engineering Simulations with Game Development Tools - OSU Capstone Project (Gaikwad, Tay, Tum)

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [Usage with ESGDT Server App](#server)
## Installation
Follow the steps given below to setup and run this project.

### Prerequisites

Before you begin, ensure you have the following installed on your machine:

- **Rust**: The programming language used for Bevy.
- **Cargo**: Rust's package manager and build system (comes with Rust).

### Step 1: Install Rust

#### For Windows

1. **Download the Installer**: Visit the [official Rust installation page](https://www.rust-lang.org/tools/install) to download the installer.
2. **Run the Installer**: Execute the downloaded file and follow the on-screen instructions. You may need to install Visual Studio C++ Build tools if prompted.
3. **Verify Installation**: Open a new command prompt and run:
   ```bash
   rustc --version

#### For macOS and Linux

1. **Use rustup**: Open your terminal and run the following command:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

  Follow On-Screen Instructions: The script will guide you through the installation process. You may need to configure your shell to add Rust to your PATH.
  Verify Installation: After installation, check the version of Rust by running:

    rustc --version

### Step 2: Clone the Bevy Repository

Clone the Bevy Repository:
(Please clone to C Drive to avoid Drive Access errors).

    git clone https://github.com/sahilg2000/ESGDT.git

Navigate to the Project Directory:
    
    cd ESGDT
    
Build the Project: Ensure that you have all dependencies installed and build the project:    
    
    cargo build

## Usage
Run the Car Simulation using Cargo:
    
    cargo run --example car

## Server
To use the Car Simulation with the Server application, please navigate to
[ESGDT Server](https://github.com/sahilg2000/ESGDT-server/)
and follow the readme instructions provided for that software.
