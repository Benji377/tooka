[![Linting](https://github.com/Benji377/tooka/actions/workflows/lint.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/lint.yml)
[![Test](https://github.com/Benji377/tooka/actions/workflows/test.yml/badge.svg)](https://github.com/Benji377/tooka/actions/workflows/test.yml)

# Tooka
<div align="center">
    <img src="assets/logo.png" alt="Tooka Logo" style="width: 45%; max-width: 400px; vertical-align: middle;">
</div>

## Introduction
**Your Automation Sidekick**

> [!CAUTION]
> This is a Work In Progress - Don't use yet!

## Usage
So basically the user uses the CLI to load and manage tasks. Tasks are defined by writing a json file that defines their name, schedule to run (cron syntax) and the modules to execute.
Each task has to be added to a specific folder in the CLI app directory where the CLI can read them and manage them. Using the dedicated CLI command add(), a file can be added as task, which means
that the file gets copied to the CLI app directory, renamed to be unique and then loaded into the system.
The CLI obviously doesn't persist across reboots but it will start again on each reboot and resume execution.

### Task Definition
A task is roughly defined like this:
```json
{
    "name": "test",
    "schedule": "* * * * 1"
    "modules": [
        "shell": {
            "command": "echo 'Hello World'"
        },
        "file": {
            "file": "text.txt"
            "on-change": "echo 'File changed'"
        },
        ...
    ]
    "output": "test_log.txt"
}
```
Where it can have multiple modules, duplicates also, with their respective attributes. The output defines where the result of each task run should be saved.
It will only save status updates like "Run completed" or not. To save the output of a module, do it in the module.

## Future

- https://www.codetriage.com/
- https://github.com/pytest-dev/pluggy
