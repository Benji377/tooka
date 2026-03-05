# Monitoring Feature

## Overview

The monitoring feature allows tooka to automatically watch folders and execute sorting at specified time intervals. This enables a "set it and forget it" workflow where folders are continuously organized in the background.

## Commands

### Add a Folder to Monitor

```bash
tooka monitor add <PATH> --interval <SECONDS> [--rules <RULE_IDS>]
```

**Arguments:**
- `<PATH>`: Path to the folder to monitor
- `--interval, -i`: Interval in seconds between automatic sorts (default: 3600 = 1 hour)
- `--rules`: Optional comma-separated list of rule IDs to apply (omit to use all rules)

**Example:**
```bash
# Monitor Downloads folder every 30 minutes
tooka monitor add ~/Downloads --interval 1800

# Monitor with specific rules
tooka monitor add ~/Documents --interval 3600 --rules "organize-pdfs,sort-images"
```

### Remove a Folder from Monitoring

```bash
tooka monitor remove <PATH>
```

**Example:**
```bash
tooka monitor remove ~/Downloads
```

### List Monitored Folders

```bash
tooka monitor list
```

Shows all currently monitored folders with their configuration (path, interval, rules).

### Show Status

```bash
tooka monitor status
```

Displays detailed status information for each monitored folder:
- Last run timestamp
- Status of last run (success/failure with details)
- Time until next scheduled run

### Run the Monitoring Daemon

```bash
tooka monitor daemon run [--check-interval <SECONDS>]
```

**Arguments:**
- `--check-interval, -c`: How often to check for due folders in seconds (default: 60)

Starts the monitoring daemon that runs continuously in the background. The daemon will:
1. Check all monitored folders at the specified check interval
2. Execute sorting on any folder that is due
3. Update the status and timestamp for each run
4. Continue running until stopped

**Example:**
```bash
# Run daemon with default 60-second check interval
tooka monitor daemon run

# Run daemon with 30-second check interval
tooka monitor daemon run --check-interval 30

# Run daemon in background (Linux/macOS)
tooka monitor daemon run > /dev/null 2>&1 &
```

### Stop the Monitoring Daemon

```bash
tooka monitor daemon stop
```

Stops a running monitoring daemon gracefully. This command:
- Locates the running daemon process
- Sends a termination signal
- Cleans up the PID file
- Confirms successful shutdown

**Example:**
```bash
tooka monitor daemon stop
```

### Check Daemon Status

```bash
tooka monitor daemon status
```

Checks if the monitoring daemon is currently running and displays its process ID (PID) if active.

**Example:**
```bash
# Check if daemon is running
tooka monitor daemon status

# Use in scripts
if tooka monitor daemon status &>/dev/null; then
    echo "Daemon is active"
fi
```

## Configuration Storage

Monitored folder configurations are stored in `~/.config/tooka/monitoring.yml`. This file is automatically created and updated as you add or remove monitored folders.

## Use Cases

1. **Downloads Folder**: Automatically organize downloaded files every 15 minutes
   ```bash
   tooka monitor add ~/Downloads --interval 900
   tooka monitor daemon run &
   ```

2. **Documents Folder**: Sort documents once per hour
   ```bash
   tooka monitor add ~/Documents --interval 3600
   tooka monitor daemon run &
   ```

3. **Project Workspace**: Monitor a project folder with specific rules every 10 minutes
   ```bash
   tooka monitor add ~/Projects/workspace --interval 600 --rules "code-files,assets"
   tooka monitor daemon run &
   ```

4. **Stop monitoring when done**:
   ```bash
   # Check if daemon is running
   tooka monitor daemon status
   
   # Stop the daemon
   tooka monitor daemon stop
   ```

## Running as a System Service

To run the monitoring daemon as a background service on Linux, you can create a systemd service:

### Create systemd service file

Create `/etc/systemd/system/tooka-monitor.service`:

```ini
[Unit]
Description=Tooka Monitoring Daemon
After=network.target

[Service]
Type=simple
User=%i
ExecStart=/usr/local/bin/tooka monitor daemon run
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

### Enable and start the service

```bash
sudo systemctl enable tooka-monitor
sudo systemctl start tooka-monitor
sudo systemctl status tooka-monitor
```

## Platform Support

The monitoring feature is implemented using cross-platform Rust threading and should work on:
- Linux
- macOS
- Windows

The daemon uses polling at specified intervals rather than filesystem events, ensuring consistent behavio

5. **Start and stop the daemon cleanly**:
   ```bash
   # Always start daemon in background
   tooka monitor daemon run &
   
   # Check status anytime
   tooka monitor daemon status
   
   # Stop gracefully when done
   tooka monitor daemon stop
   ```

6. **Prevent duplicate daemons**: The system automatically prevents running multiple daemons simultaneouslyr across all platforms.

## Best Practices

1. **Choose appropriate intervals**: Balance between responsiveness and system resources
   - For frequently changing folders: 300-900 seconds (5-15 minutes)
   - For occasional changes: 1800-3600 seconds (30-60 minutes)

2. **Use specific rules when possible**: If you only need certain rules for a folder, specify them with `--rules` to improve performance

3. **Monitor daemon logs**: Check logs in `~/.local/share/tooka/logs/` for troubleshooting

4. **Test first**: Verify your rules work correctly with manual `tooka sort` before setting up monitoring
