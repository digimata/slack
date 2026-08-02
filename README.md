# slack

Operate Slack conversations from the command line: read channels, send
messages as yourself, search, react — across multiple workspaces.

```sh
cargo install --path .
```

## Authentication

Profiles live in `~/.config/slack/config.json` (mode 0600). Each profile is
one workspace credential. Two operator paths, one bot path:

| Token | What it is | When to use it |
|---|---|---|
| `xoxp-` | User token from a private internal Slack app | Workspaces you control. Supported and durable. |
| `xoxc-` + `d` cookie | Browser session pair | Workspaces where you can't install an app (client workspaces). Unsanctioned; dies with the browser session. |
| `xoxb-` | Bot token | Explicit bot automation only. |

Both operator paths post as *you*. After every send the CLI verifies the
message author equals the profile's `auth.test` user id.

### Adding a profile

```sh
slack auth add work            # hidden prompt for the token (and cookie if xoxc)
slack auth list
slack auth use work            # set default
slack auth status              # verify identity via auth.test
```

Non-interactive: `printf 'xoxc-…\nxoxd-…\n' | slack auth add work --token-stdin`
(token on line 1, cookie on line 2). One-off/CI override: `SLACK_TOKEN`
(+ `SLACK_COOKIE` for xoxc), no profile needed.

### Getting an xoxp token (your workspaces)

Create a private app from `examples/app-manifest.yaml` at
<https://api.slack.com/apps>, install it to the workspace, and copy the
**User OAuth Token** (`xoxp-…`) from *OAuth & Permissions*.

### Getting the xoxc/xoxd pair (client workspaces)

1. Open the workspace in a browser and sign in.
2. DevTools → Network → filter `api/` → click any request.
3. Token: in the request payload/form data, the `token` field (`xoxc-…`).
4. Cookie: in the request Cookie header, the `d=` value (`xoxd-…`, keep it
   URL-encoded exactly as shown).

The pair is invalidated when the browser session ends or is signed out.

## Usage

```sh
slack channel list --types public_channel,private_channel
slack channel info '#general'

slack message list '#general' --limit 50
slack message list '@Ada Lovelace'                 # DM history
slack message thread '#ops' 1754160000.000100
slack message send '#ops' 'deploy done'
printf 'multi\nline' | slack message send '@ada'
slack message send '#ops' 'reply' --thread 1754160000.000100
slack message send '#ops' 'check' --dry-run        # shows posting identity
slack message edit '#ops' 1754160000.000100 'fixed wording'
slack message delete '#ops' 1754160000.000100      # prompts; --yes to skip
slack message permalink '#ops' 1754160000.000100

slack search messages 'from:andrew in:#shipping regression'
slack user list
slack user info '@ada'
slack reaction add '#ops' 1754160000.000100 white_check_mark

slack api conversations.members channel=C01234567 limit=200
slack api chat.postMessage --data payload.json
```

Global flags: `--profile NAME`, `--json` (stable machine output),
`--no-cache`, `--verbose`.

### Targets

IDs always work (`C…`, `D…`, `U…`). `#name` and bare names resolve against
the channel list; `@name` resolves against the member directory (display,
handle, or real name). Ambiguity is an error listing candidates, never a
guess. Directory lookups cache under `~/.cache/slack/<profile>/` for 5
minutes.

### Exit codes

`0` success · `2` usage/ambiguity · `3` auth/config · `4` Slack API
rejection · `5` transport · `6` rate limit not retried within budget.

## Security notes

- Tokens are stored 0600 in your home directory, never accepted as command
  arguments, and never printed.
- 429s retry within a 60s budget honoring `Retry-After`. Mutations are never
  replayed after an ambiguous transport failure.
- The `xoxc` path automates your own session against Slack's internal-use
  API surface; Slack does not sanction it. Prefer `xoxp` where you can
  install an app.

## Name collision

Slack ships an official app-development CLI also named `slack`. This tool
does not overlap with it (no app scaffolding, deploys, or event listeners).
If you ever need both, alias the official one to `slack-dev`.
