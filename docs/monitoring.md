# Monitoring

The monitoring daemon sorts folders automatically at regular intervals.

## Set up

```bash
tooka monitor add ~/Downloads --interval 3600        # every hour, all rules
tooka monitor add ~/Desktop -i 600 --rules screenshots
tooka monitor list
```

Paths are stored as absolute paths. A folder can only be added once.

## Run the daemon

```bash
tooka monitor daemon run
```

The daemon runs in the foreground of that terminal. It checks every 60
seconds (`--check-interval` to change) which folders are due and sorts
them, exactly like `tooka sort` would. A folder that was never sorted is
sorted right away. Folders added or removed while the daemon runs are
picked up automatically.

To keep it running after you log out, start it with your system's tools,
for example a systemd user service on Linux:

```ini
# ~/.config/systemd/user/tooka.service
[Unit]
Description=Tooka monitoring daemon

[Service]
ExecStart=%h/.cargo/bin/tooka monitor daemon run
Restart=on-failure

[Install]
WantedBy=default.target
```

```bash
systemctl --user enable --now tooka
```

(Adjust `ExecStart` to where `tooka` is installed: `which tooka`.)

## Stop it

Press Ctrl+C in its terminal, or from anywhere:

```bash
tooka monitor daemon stop
```

The daemon stops at the next safe point. A sort in progress is undone, just
as when you press Ctrl+C during `tooka sort`; the folder is sorted again at
its next turn. Only if the daemon does not react at all is it stopped by
force, after which `tooka recover` may be needed.

Only one daemon can run at a time. `tooka monitor daemon status` tells
whether it is running.

## Check results

```bash
tooka monitor status
```

shows for each folder when it was last sorted, how many files changed, or
why it failed. Keep in mind:

- With the default `on_conflict: abort`, a single conflicting file stops the
  whole run, so nothing in that folder gets sorted until you resolve it.
  Consider `on_conflict: skip` or `rename` for rules used by the daemon
  (see [Conflicts](rules.md#conflicts-on_conflict)).
- If a manual `tooka sort` is running when a folder is due, the daemon
  simply tries again at the next check.
- If a sort was interrupted (crash, power loss), the daemon waits until you
  run `tooka recover`.
- Relative `to` folders (`./...`) are resolved from the folder the daemon was
  started in; prefer absolute or `~/...` paths in rules used by the daemon.
