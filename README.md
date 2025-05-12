[![Linting](https://github.com/Benji377/tooka/actions/workflows/lint.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/lint.yml)
[![Test](https://github.com/Benji377/tooka/actions/workflows/test.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/test.yml)

# Tooka
<div align="center">
    <img src="assets/logo-banner.png" alt="Tooka Logo" style="width: 70%; max-width: 1280px; vertical-align: middle;">
</div>


**Your Automation Sidekick**

---

## 🧠 What is Tooka?

**Tooka** is a lightweight and modular command-line tool that lets you define and automate tasks using simple JSON files. Think of it as a customizable automation assistant: you write tasks in JSON, define their behavior using modules, and let Tooka handle execution and output—all from your terminal.

Each task consists of one or more **modules**, which are executed in sequence. Modules act like plugins: reusable, composable, and easily extended.

---

## 🚀 Installation

1. Download the appropriate binary for your operating system from the [Releases page](https://github.com/Benji377/tooka/releases).
2. (Optional) Add it to your system's PATH for global access.

> Other installation methods (e.g., Homebrew, Winget, etc.) coming soon!

---

## 🛠️ Usage

### 1. Define a Task

Create a JSON file (e.g., `test_task.json`) with your task definition:

```json
{
  "name": "shell-task",
  "description": "A task that runs a shell command",
  "modules": [
    {
      "name": "shell",
      "config": {
        "command": "echo 'hello from shell'"
      }
    }
  ]
}
```

### 2. Add the Task

```bash
tooka add test_task.json
```

### 3. Run the Task

```bash
tooka run shell-task
```

🎉 Done! Tooka will execute the defined modules and output the result.

> 📚 For advanced usage and task chaining, check out the [Wiki](https://github.com/Benji377/tooka/wiki).

---

## 🧩 Extending with Modules

Modules are the building blocks of Tooka. You can think of them as plugins that define specific functionality (e.g., shell commands, HTTP requests, file processing).

Want to build your own module? Check the [Contributing Guide](https://github.com/Benji377/tooka/wiki/Contributing) for how to get started.

---

## 🤝 Contributing

We welcome contributions of all kinds—bug fixes, documentation improvements, and especially **new modules**!

Modules can be developed and maintained independently, allowing Tooka to remain lightweight and extensible.

If you'd like to get involved:

* Read the [Contributing Guide](https://github.com/Benji377/tooka/wiki/Contributing)
* Submit an issue or feature request
* Fork the repo and open a pull request

---

## 📄 License

This project is licensed under the [GPLv3 License](LICENSE).

