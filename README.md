# Discord Music Bot

**Languages:** English | [Русский](README.ru.md)

A Discord music bot written in Rust using Serenity, Poise, and Songbird.

## Features

- Play YouTube links, search queries, playlists, and multiple links in one request.
- Play an audio attachment or a file available on the bot host.
- Per-server queue with automatic progression when a track ends.
- Repeat track, repeat queue, shuffle, volume control, seek, and playback history.
- A public control panel with buttons for previous, pause/resume, skip, stop, clear, queue, repeat, shuffle, and volume.
- Only one current panel per server. The panel follows control activity between text channels; older panels are disabled.
- Recover after the bot is removed from voice: clear the interrupted track and queue, then wait for a new `/play`.
- Leave voice automatically after 10 minutes with no current or queued tracks. `/leave` disconnects immediately.
- While playing or waiting with a non-empty queue, the current voice channel has priority. During an empty idle wait, a new `/play` can move the bot to the caller's channel.

Queue and playback state are held in memory and are lost when the bot restarts.

## Commands

| Command | Arguments | Description |
|---|---|---|
| `/play` | `query` | Play a YouTube URL, search query, playlist, or multiple URLs. |
| `/play_local` | `file` or `path` | Play an attached audio file or a file on the bot host. A host path is not a path on the user's computer. |
| `/nowplaying` | — | Show the current track and playback progress. |
| `/queue` | `page` (optional) | Show the current track and a page of queued tracks. |
| `/skip` | — | Skip the current track. |
| `/previous` | — | Return to the previous track in playback history. |
| `/pause`, `/resume` | — | Pause or resume playback. |
| `/loop` | `mode`: `Off`, `Track`, or `Queue` (optional) | Set or cycle the repeat mode. |
| `/shuffle` | — | Shuffle upcoming tracks. |
| `/volume` | `level`: `0–200` | Set volume as a percentage. |
| `/seek` | `position` | Seek by seconds or `mm:ss` (for example, `90` or `1:30`). |
| `/replay` | — | Restart the current track. |
| `/remove` | `index` | Remove a queued track by its 1-based position. |
| `/clear` | — | Clear upcoming tracks without stopping the current track. |
| `/stop` | — | Stop playback and clear the queue; the bot leaves after the idle timeout. |
| `/leave` | — | Disconnect from voice immediately and clear playback state. |

The bot registers slash commands. The command functions also declare Poise prefix support, but this project does not configure a message prefix, so use slash commands and the control panel.

## Requirements

- Rust and Cargo
- `yt-dlp` and FFmpeg available in `PATH`
- `libopus` and the platform's usual build tools

### macOS

```bash
brew install ffmpeg yt-dlp opus
```

### Ubuntu / Debian

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev ffmpeg yt-dlp libopus-dev
```

## Configure and run

1. Create a bot application in the [Discord Developer Portal](https://discord.com/developers/applications), add a bot, and copy its token.
2. Create `.env` in the project root:

   ```env
   DISCORD_TOKEN=your_bot_token
   ```

3. In **Bot → Privileged Gateway Intents**, enable the intents requested by this bot: **Presence**, **Server Members**, and **Message Content**. The application currently requests all gateway intents.
4. Invite the bot with the `bot` and `applications.commands` OAuth2 scopes. Grant the channel permissions it needs: View Channels, Send Messages, Embed Links, Connect, and Speak. Administrator permission is not required.
5. Run locally or build a release binary:

   ```bash
   cargo run
   # or
   cargo build --release
   ./target/release/discord-music-bot
   ```

## Run with systemd

Adjust the paths and service user for your VPS. This example matches a deployment under `/root/rustDiscordBot`:

This mirrors the current VPS unit and runs as `root`. For a new production deployment, use a dedicated system user and grant it access only to the project directory.

```ini
[Unit]
Description=Discord Music Bot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/rustDiscordBot
ExecStart=/root/rustDiscordBot/target/release/discord-music-bot
Restart=always
RestartSec=5
EnvironmentFile=/root/rustDiscordBot/.env

[Install]
WantedBy=multi-user.target
```

Save it as `/etc/systemd/system/discord-bot.service`, then build and start the service:

```bash
cd /root/rustDiscordBot
cargo build --release
sudo systemctl daemon-reload
sudo systemctl enable --now discord-bot.service
sudo systemctl status discord-bot.service
```

View logs with `sudo journalctl -u discord-bot.service -f`.

## Checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
