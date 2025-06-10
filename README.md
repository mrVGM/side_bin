# SideBin

**SideBin** is a lightweight productivity tool for **Windows** that simplifies file handling and boosts workflow efficiency. Think of it as a temporary, accessible "shelf" for your files — perfect for when you need to move or reference files between folders without juggling multiple File Explorer windows.

> **Note:** SideBin is currently available **only on Windows**.

---

## 🧰 Features

- 🪟 Small, collapsible window that stays on top
- 🖱️ Expands on mouse hover for quick access
- 📂 Drag and drop files into slots for temporary storage
- 🧲 Drag files back out to any location or app when needed
- 🔒 Keeps references to files (does not copy or move them)
- 🎯 Ideal for multitasking, file transfers, or organizing workspaces

---

## 💡 Use Case

Say you're working on organizing files across different directories — instead of opening several Explorer windows, just drag your files into **SideBin**. When you're ready to place them somewhere else, drag them out again from SideBin. Fast, simple, and intuitive.

---

## ⚙️ Powered by Tauri

SideBin is built with [Tauri](https://tauri.app/), a modern framework for creating fast, secure, and lightweight desktop applications using web technologies like HTML, CSS, and JavaScript — with a native Rust backend.

Tauri makes SideBin:

- ✅ Extremely lightweight
- ✅ Secure and performant
- ✅ Easy to update and maintain

---

## 🛠️ Configuration

You can customize **SideBin's** appearance and behavior using two optional files:

- **config.json** – controls window size, character limit per filename, anchor point...
- **style.css** – allows limited visual customization (e.g., colors, spacing)

Place these files in **either** of the following locations:

- The **same directory** as the application executable  
- A `.side_bin` folder in your **home directory**

> 📁 You can find example configuration files in the `sample_config` directory of the project.

This setup gives you flexibility to apply either per-instance or user-wide settings.

---

## 🛠️ Build from Source

To build **SideBin** from source, follow these steps:

### 1. Build the `fs_monitor` static library (C++)

- Ensure you have **CMake** and **Clang** installed.
- Open a command prompt and navigate to the `fs_monitor` directory.
- Run the `build.bat` script to compile the library.
- After a successful build, copy `fs_monitor.lib` from the `ready` folder into `src-tauri\lib`.

### 2. Build the Tauri application

- Make sure you have the following installed:
  - **Rust**
  - **Tauri CLI**
  - A JavaScript runtime (e.g., [Bun](https://bun.sh/))

- From the project root, run:

  ```bash
  bun install
  bun run tauri build
  
After building, you'll find the compiled application in the `src-tauri/target/release` directory.

---

