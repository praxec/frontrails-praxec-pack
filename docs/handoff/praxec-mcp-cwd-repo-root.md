# Handoff → praxec/flowgate team: `kind: mcp` children ignore the run's `repo_root`

**Repo:** `mcp-flowgate` · **Component:** `crates/praxec-executors/src/mcp.rs`
**Severity:** medium (correctness surprise + a "server down" failure mode on slow filesystems)
**Filed by:** FrontRails pack maintainers, 2026-07-28. Line numbers are from the tree we read that day — re-anchor before editing.

---

## TL;DR

A `kind: mcp` connection is spawned **without a working directory**, so the child MCP server
inherits the **gateway process's cwd** — it does *not* honor the workflow's `repoRoot` selector
(`$.run.repo_root`) the way the `script`, `cli`, and path-grounded executors do. Any relative path
the connection passes to the child (e.g. our pack's `env: { STRUCTUREOS_WORKSPACE_ROOT: "." }`)
therefore resolves against wherever the gateway was launched, not the repo the run targets.

Combined with the 30s connect idle-timeout, this produced a **false "mcp down"**: with the gateway's
cwd on a Windows `drvfs` mount, a downstream server's cold-start storage work (a full-tree `statx`
walk) ran long and silent, and the connection was dropped before the `initialize` handshake landed.

We've shipped a downstream mitigation (`startupTimeoutMs` in the pack) and fixed our own server
(FrontRails 0.0.21 no longer walks the tree before the handshake), but the **cwd/`repo_root` gap is
a gateway-side issue** and is what this handoff is about.

## Root cause

`crates/praxec-executors/src/mcp.rs` (spawn path, ~L325–339):

```rust
let mut cmd = tokio::process::Command::new(command);
cmd.args(&conn.args);
for (k, v) in &conn.env { cmd.env(k, v); }
cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
let mut child = cmd.spawn() ...;          // <-- no cmd.current_dir(...)
```

There is **no `cmd.current_dir(...)`**. Contrast the sibling executors, which DO ground to the run's
root:

- `crates/praxec-executors/src/script.rs:327` → `cmd.current_dir(wd)`
- `crates/praxec-executors/src/cli.rs:124`    → `cmd.current_dir(wd)`
- `crates/praxec-executors/src/workflow.rs:271–303` → per-spawn `repoRoot` override resolves
  `$.run.repo_root`; `path_grounding.rs` treats `$.run.repo_root` as the mandatory run root.

So the plumbing for "run is rooted at `repo_root`" already exists; the `mcp` executor just doesn't
consume it. Two consequences:

1. **Silent mis-rooting.** A downstream server that persists state or scans "the workspace" (ours
   writes `.frontrails/{structure,intent,security}/…` and reads `WORKSPACE_ROOT`) operates on the
   *gateway's* cwd, not the repo the operator selected via `repoRoot`. State can land in the wrong
   tree; analysis can target the wrong tree.
2. **Launch-context-dependent latency.** If the gateway's cwd is a slow mount, every downstream
   server's cwd-rooted cold-start work is slow — regardless of which (fast, local) repo the run
   actually targets.

### Why it surfaced as "down" (the idle-timeout interaction)

`mcp.rs` establishment uses an **inactivity** window, not a wall-clock budget:

- `DEFAULT_IDLE_TIMEOUT_MS = 30_000` (L110); `startupTimeoutMs` widens *just* establishment (L131–139, FB-7).
- The clock is bumped on **every child stdout AND stderr byte** (L361 for stderr drain, L377 for the
  stdout transport), so a server that logs progress stays alive.
- `with_idle_timeout(startup, &clock, ServiceExt::serve(handler, transport))` (L379) wraps the handshake.

The trip condition is therefore: **the child emits nothing on stdout or stderr for 30 continuous
seconds.** A downstream server doing a silent, unbounded, cwd-rooted filesystem walk on cold start
hits exactly that on a slow mount. The gateway can't distinguish "silently working" from "hung," so
it (correctly, by its contract) drops the connection.

## Reproduction

1. Put a large repo on a slow filesystem (WSL2 `/mnt/c/...` drvfs; `statx` ≈ 10ms/file there).
2. Launch a `kind: mcp` connection whose child does cwd-rooted work on startup, with the **gateway's
   cwd on that mount**.
3. Observe: child spawns, prints a line or two to stderr, then goes silent during the walk; at 30s
   the gateway reports the connection down. Same binary from an interactive shell rooted on a native
   path (`/home/...`) handshakes in < 0.1s.

(In our case the walk was structureos's cache-staleness `statx` sweep. We measured 32s idle-timeout
on drvfs vs 0.06s after moving the walk off the handshake path.)

## Recommended fixes (gateway side)

1. **Root `kind: mcp` children at the run's `repo_root`** — the core ask. Set
   `cmd.current_dir(run_repo_root)` (resolved the same way `workflow.rs` resolves the `repoRoot`
   override / `$.run.repo_root`) before `spawn()`, and/or expose `repo_root` so a connection can bind
   its workspace-root env to it instead of a bare `"."`. This makes MCP servers honor `repoRoot` like
   every other executor and removes the "state lands in the gateway's cwd" surprise.
2. **Make a silent-but-alive establishment observably different from a hang** (optional, defense in
   depth). E.g. document/encourage `startupTimeoutMs`, or emit a one-line "connecting to `<name>`…"
   breadcrumb so operators see *which* connection is slow rather than a bare timeout.
3. **Doc note:** the `command:` spawn resolves via the gateway process's `PATH` (bare-name commands
   fail with ENOENT under a GUI/service launcher that lacks `~/.cargo/bin`). A short line in the
   connections docs would save the next person the PATH-vs-hang confusion (an ENOENT fails instantly;
   a slow-fs handshake fails at the idle timeout — different symptoms, different fixes).

## What we already did downstream (so you can see the full picture)

- **Pack cushion:** `startupTimeoutMs: 120000` on all four `*os` connections (`frontrails.yaml`,
  v1.3.2) and absolute-path `command:` (PATH-independent spawn).
- **Server fix:** FrontRails 0.0.21 — `structureos-mcp` warms its cache in a background task instead
  of walking the tree before the handshake, so establishment is O(1) regardless of filesystem.

Neither addresses item (1) — MCP servers still root at the gateway's cwd. That's yours. Thanks!
