# Discord Music Bot
Bot for playing music in Discord voice channels.

### Features
- Plays music from every source supported by [yt-dlp](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md) (YouTube, Soundcloud, Reddit, etc.)
- Uses slash commands
- Has queue
- Has cache
- Supports local audio files
- Leaves voice channel after song/queue ends

## How to run
### Tokens
To run this bot you have to create Discord Application on [Discord Developer portal](https://discord.com/developers/applications) and add its API token to PATH. Example for Linux:
```bash
export DISCORD_TOKEN="your_token_here"
```
Example for Windows cmd:
```cmd
set DISCORD_TOKEN="your_token_here"
```

### yt-dlp
Bot uses [yt-dlp](https://github.com/yt-dlp/yt-dlp) so you have to install yt-dlp/download yt-dlp binary and pass its path to bot using `--yt-dlp` argument. Example for Linux:
```bash
./musicbot --yt-dlp /path/to/yt-dlp
```
Example for Windows:
```cmd
musicbot.exe --yt-dlp C:\path\to\yt-dlp.exe
```

### Cookies
You have to provide Netscape formatted cookies file for yt-dlp to be able to play age-restricted videos from YouTube. You can use [cookies.txt](https://addons.mozilla.org/en-US/firefox/addon/cookies-txt/). Pass path to exported file using `--cookies` argument.

### Local audio files
You can pass path to directory with local audio files using `--local_audio` argument. Bot will be able to play audio files from this directory using `play_local` command.


## How to build
You need to have [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed
1. Clone repository
2. Run `cargo build --release`
3. Built binary will be in `target/release` directory