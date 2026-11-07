# Commands

Run `tooka <command> --help` for the same information in the terminal.

## Rules

| Command | Description |
|---|---|
| `tooka template [--output FILE]` | Writes a commented example rule (default `rule_template.yaml`). Never overwrites an existing file. |
| `tooka validate FILE` | Checks a rule file: YAML structure, regular expressions, globs, dates, action order. Exits with an error if anything is wrong. |
| `tooka add PATH [--overwrite]` | Adds the rule(s) from a YAML file, or from every YAML file in a folder. Rules are validated first. `--overwrite` replaces rules with the same `id`. |
| `tooka list` | Lists all rules and whether they are enabled. |
| `tooka toggle ID` | Enables or disables a rule. |
| `tooka export ID [--output FILE]` | Prints a rule as YAML, or writes it to a file (never overwrites). |
| `tooka remove ID` | Deletes a rule. |

## Sorting

```text
tooka sort [--source FOLDER] [--rules ID,ID,...] [--dry-run]
           [--report json|csv|pdf] [--output FOLDER]
```

| Option | Meaning |
|---|---|
| `--source` | Folder to sort (default: the source folder from the [configuration](configuration.md)) |
| `--rules` | Only use these rules (comma-separated IDs) |
| `--dry-run` | Show the plan, change nothing |
| `--report` | Also write a report: `tooka_report.json`, `.csv` or `.pdf` |
| `--output` | Folder for the report (default: the current folder) |

`tooka recover` undoes a sort that was interrupted by a crash, a killed
process or a power loss. See [Sorting](sorting.md#interrupted-sorts).

## Monitoring

| Command | Description |
|---|---|
| `tooka monitor add PATH [-i SECONDS] [--rules ID,...]` | Sorts `PATH` automatically every `SECONDS` (default 3600). |
| `tooka monitor remove PATH` | Stops monitoring a folder. |
| `tooka monitor list` | Lists monitored folders. |
| `tooka monitor status` | Shows when each folder was last sorted and how it went. |
| `tooka monitor daemon run [-c SECONDS]` | Runs the daemon in the foreground; checks for due folders every `SECONDS` (default 60). |
| `tooka monitor daemon stop` | Stops a running daemon. |
| `tooka monitor daemon status` | Tells whether the daemon is running. |

See [Monitoring](monitoring.md).

## Other

| Command | Description |
|---|---|
| `tooka config --show` | Shows the configuration. |
| `tooka config --locate` | Shows where the configuration file is. |
| `tooka config --reset` | Resets the configuration to its defaults. |
| `tooka completions SHELL` | Prints shell completions (bash, zsh, fish, powershell, elvish). |
