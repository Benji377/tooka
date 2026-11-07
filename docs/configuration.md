# Configuration

```bash
tooka config --locate   # where the configuration file is
tooka config --show     # what is in it
tooka config --reset    # back to the defaults
```

## Files and folders

Tooka uses the standard application folders of your system (on Linux
`~/.config/tooka` and `~/.local/share/tooka`; on macOS and Windows the
usual application-support folders, shown by `tooka config --locate`).

| File | Folder | Content |
|---|---|---|
| `tooka.yaml` | config folder | the configuration (see below) |
| `monitoring.yml` | config folder | monitored folders and their last results |
| `rules.yaml` | data folder (by default) | your rules; manage them with `tooka add` etc. |
| `logs/` | data folder (by default) | log files |
| `sort-journal.jsonl` | data folder | only exists while a sort runs or after it was interrupted |

## `tooka.yaml`

```yaml
version: 0
source_folder: /home/me/Downloads   # sorted when no --source is given
rules_file: /home/me/.local/share/tooka/rules.yaml
logs_folder: /home/me/.local/share/tooka/logs
```

You can edit these paths by hand. The default source folder is your
system's Downloads folder.

## Environment variables

Useful for testing, portable setups, or several independent setups:

| Variable | Effect |
|---|---|
| `TOOKA_CONFIG_DIR` | folder for `tooka.yaml` and `monitoring.yml` |
| `TOOKA_DATA_DIR` | folder for the sort journal, and the default for the rules file and logs of a new configuration |
| `TOOKA_SOURCE_FOLDER` | default source folder of a new configuration |

"New configuration" means: these only take effect when `tooka.yaml` is
created (first run, or `tooka config --reset`); afterwards the paths in
`tooka.yaml` are used.

## Logs

The logs folder contains:

- `main.log`: everything Tooka does, useful for bug reports,
- `ops/<date>.log`: one line per file operation (moved, copied, renamed,
  deleted, rolled back), useful to find out where a file went.

## Upgrading from 1.1.x

Configuration, rules and monitoring settings from older versions keep
working. Before 1.2.0 the monitoring settings were always stored in
`~/.config/tooka/monitoring.yml`, also on macOS and Windows; they are moved
to the config folder automatically.
