# portopsy

> Find who owns a port, understand what started it, and resolve it safely.

`portopsy` is a Linux-first CLI for the moment your development server says:

```text
Error: listen EADDRINUSE: address already in use
```

Instead of stitching together `ss`, `lsof`, `ps`, `pwdx`, `systemctl`, and `docker ps`, run one command.

![portopsy terminal demo](demo.gif)

<p align="center"><sub>Live diagnostic output followed by machine-readable JSON output.</sub></p>

## Why portopsy?

A PID alone is rarely enough. `portopsy` connects the socket to the context you need to decide what to do:

- Which process owns the port?
- What exact command started it?
- Which project directory is it running from?
- Is it inside Docker, Podman, or systemd?
- What is the safest way to release the port?

It is local-first, has no daemon, requires no API key, and does not send diagnostics to a cloud service.

## Quick start

### Requirements

- Linux or WSL2
- Rust 1.85 or newer when building from source
- A mounted `/proc` filesystem

```bash
# Install from the project directory
cargo install --path .

# Inspect a development port
portopsy 3000
```

If the binary is not found after installation, add Cargo's bin directory to your shell path:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Build a release binary

```bash
cargo build --release
./target/release/portopsy 3000
```

## What it shows

For matching TCP and UDP sockets, portopsy reports:

| Layer | Details |
| --- | --- |
| Socket | Protocol, local address, port, state, inode, UID |
| Process | PID, full command line, executable path |
| Context | Working directory, user, parent process chain |
| Runtime | Docker/Podman container ID and systemd unit or scope |
| Resolution | Safe commands to review before taking action |

Example:

```text
portopsy
Inspecting port 3000
────────────────────────────────────────────────────────
1 socket match(es)

TCP://0.0.0.0:3000  LISTEN
  protocol     TCP
  endpoint     0.0.0.0:3000
  inode        18547  uid 1000

  process      node ./server.js (PID 1842)
  executable   /usr/bin/node
  user         pratyush (uid 1000)
  cwd          /home/pratyush/work/client-app
  parents      node (1842) -> bash (1821) -> tmux: server (1800)

  SAFE ACTIONS  (not executed)
    $ kill -TERM 1842  # graceful process stop
```

## Usage

```bash
# Accepts 3000, :3000, or host:port syntax
portopsy 3000
portopsy :3000
portopsy 0.0.0.0:3000

# Protocol filters
portopsy --tcp 8080
portopsy --udp 5353
portopsy --all 3000

# Explicit port aliases
portopsy --port 3000
portopsy -p 3000
portopsy --port=3000

# Protocol-prefixed input
portopsy tcp://:3000
portopsy udp://:5353

# Machine-readable output for scripts and agents
portopsy --json :3000
```

### Port syntax

All of these resolve to the same port:

```text
portopsy 3000
portopsy :3000
portopsy 0.0.0.0:3000
portopsy --port 3000
portopsy -p 3000
portopsy --port=3000
```

## Interactive resolution

The default command is read-only. To execute one carefully guarded resolution action, opt in explicitly:

```bash
portopsy --resolve --tcp :3000
# shorthand
portopsy -r --tcp :3000
```

The standard diagnostic appears first. Then portopsy presents a stable menu:

```text
[1] Stop systemd service
[2] Stop container
[3] Send SIGTERM
[4] Cancel
```

Unavailable actions remain visible and are marked clearly instead of changing the menu numbers:

```text
[2] Stop container
    unavailable for this process
```

Before executing options 1, 2, or 3, portopsy:

1. Re-checks `/proc` to ensure the PID still exists.
2. Verifies that the executable and command still match the inspected process.
3. Shows the exact command or signal action.
4. Requires you to type exactly `yes`.
5. Executes only the selected action.

### Resolution behavior

- **Stop systemd service** uses `systemctl stop <unit>` or `sudo systemctl stop <unit>` when needed.
- **Stop container** checks for `docker` and `podman` in `$PATH`.
- **Send SIGTERM** uses the native Linux `kill(2)` system call.
- After SIGTERM, portopsy waits up to 1.5 seconds for the process to exit.
- If it is still alive, portopsy asks before sending SIGKILL:

```text
Process did not exit after SIGTERM. Send SIGKILL (kill -9 1842)? [y/N]
```

SIGKILL is only sent when you answer `y` or `Y`.

Resolution requires interactive stdin and stdout. It will not run in a pipe, CI environment, or JSON mode.

## Terminal output

Human output uses ANSI colors automatically in interactive terminals:

- Cyan and blue identify the tool, sections, and labels.
- Green highlights healthy socket states and commands.
- Yellow highlights safe actions and warnings.
- Dim text is used for supporting context such as parent chains and inodes.

Disable colors when needed:

```bash
portopsy --no-color :3000
portopsy -n :3000
NO_COLOR=1 portopsy :3000
```

JSON output never contains ANSI escape sequences.

## Safety and permissions

Portopsy is read-only by default:

- It does not kill processes unless `--resolve` is explicitly selected and confirmed.
- It does not stop containers unless `--resolve` is explicitly selected and confirmed.
- It does not restart services.
- It does not modify files or network configuration.
- It does not send data to a cloud service.
- It never runs commands through `sh -c` or `bash -c`.

A normal user can inspect processes it owns. Linux may hide command lines, working directories, file descriptors, or PIDs belonging to another user. If context is hidden, try:

```bash
sudo portopsy :3000
```

If a confirmed action fails with a permission constraint, portopsy prints:

```text
Action failed due to permission constraints. Try running with elevated privileges:
  $ sudo portopsy --resolve
```

Only run elevated commands when you trust the binary and the source code you built.

## Architecture

The Linux implementation uses standard kernel interfaces and has no runtime service:

1. Read `/proc/net/tcp`, `/proc/net/tcp6`, `/proc/net/udp`, and `/proc/net/udp6`.
2. Filter entries by the requested port.
3. Scan `/proc/<pid>/fd` for matching socket inodes.
4. Read process context from `/proc/<pid>/cmdline`, `cwd`, `exe`, `status`, and `cgroup`.
5. Render human-readable output or JSON.
6. Execute guarded actions only after explicit confirmation.

The Rust CLI itself uses the standard library only.

## Development

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

Run a local demo in WSL:

```bash
python3 -m http.server 3000 &
portopsy 3000
```

The VHS source for the included demo is available in [`demo.tape`](demo.tape).

## Roadmap

- macOS support using `lsof` and native process APIs
- Unix-domain socket lookup
- Better Docker and Podman runtime-name resolution
- `ss` fallback for restricted `/proc/net` environments
- Shell completions
- Homebrew and release binaries
- More resolution providers and platform support

## License

MIT. See [`LICENSE`](LICENSE).