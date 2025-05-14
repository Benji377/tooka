[![Linting](https://github.com/Benji377/tooka/actions/workflows/lint.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/lint.yml)
[![Go Report Card](https://goreportcard.com/badge/github.com/Benji377/tooka)](https://goreportcard.com/report/github.com/Benji377/tooka)

# Tooka

<div align="center">
    <img src="assets/logo-banner.png" alt="Tooka Logo" style="width: 70%; max-width: 1280px; vertical-align: middle;">
</div>

---

**Tooka** – *"Your tasks, your way. Organize, manage, and conquer!"*

Tooka is a cross-platform TUI (Terminal User Interface) task manager. It’s fast, easy to use, and beautifully minimalistic. All data is stored locally on your machine, giving you full control over your tasks — no cloud, no tracking.

## ✨ Features

* 🧘 Minimal and beautiful UI built with [BubbleTea](https://github.com/charmbracelet/bubbletea) and [Lipgloss](https://github.com/charmbracelet/lipgloss)
* ✅ Add, edit, and toggle tasks with just a few keypresses
* 📅 Tasks include title, description, due date, priority, and completion state
* 🧠 Simple local storage (`$HOME/.tooka/`)
* 🔒 100% offline, no external dependencies
* 💾 Easy sync support: just back up a single JSON file
* 🛠 Logs available for debugging

---

## 📦 Installation

1. Visit the [Releases Page](https://github.com/Benji377/tooka/releases) and download the binary for your OS.
2. Rename it (optional):

   ```sh
   mv tooka-linux-amd64 tooka
   ```
3. Make it executable (if needed):

   ```sh
   chmod +x tooka
   ```
4. Run it:

   ```sh
   ./tooka
   ```

> ℹ️ Make sure your terminal window is wide enough for the interface to render properly — cramped UIs are no fun.

---

## 📂 File Locations

| Purpose      | Path                  |
| ------------ | --------------------- |
| Task storage | `~/.tooka/tasks.json` |
| Logs         | `~/.tooka/logs/`      |

You can back up or sync your tasks by copying `tasks.json` across devices.

---

## 🧭 Usage

Once Tooka is running, you can interact with it using keyboard shortcuts. The available commands are shown at the bottom of the UI.

Each task can have:

* Title
* Description (optional)
* Due date
* Completion status
* Priority (e.g., low/medium/high)

Internally, every task also includes:

* A unique ID
* Creation timestamp

> 🧠 Advanced users can edit `tasks.json` manually — just be sure to maintain unique IDs to avoid errors.

---

## 💬 Contributing

Got ideas? Found a bug? Want to improve the UI?

* 💡 Head over to the [Discussions](https://github.com/Benji377/tooka/discussions) tab and share your thoughts.
* 🐛 For code-related issues, please [open an issue](https://github.com/Benji377/tooka/issues).
* 🤝 Pull requests are *very* welcome — especially for UI improvements or new features.

---

## 🛠 Tech Stack

* [Go](https://golang.org)
* [BubbleTea](https://github.com/charmbracelet/bubbletea)
* [Lipgloss](https://github.com/charmbracelet/lipgloss)
* [Charm ecosystem](https://github.com/charmbracelet)

---

## 📜 License

Tooka is licensed under the GPLv3 License. See [LICENSE](./LICENSE) for details.
