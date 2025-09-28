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
To run this bot you have to create Discord Application on [Discord Developer portal](https://discord.com/developers/applications) and put its API token in .env file. File should look like this:
```
DISCORD_TOKEN=your_token_here
```

### yt-dlp
Bot uses [yt-dlp](https://github.com/yt-dlp/yt-dlp) so you have to download yt-dlp binary and put it in the same directory as bot binary. Bot expects binary to be named `yt-dlp`.

### Cookies
You have to provide Netscape formatted cookies file for yt-dlp to be able to play age-restricted videos from YouTube. You can use [cookies.txt](https://addons.mozilla.org/en-US/firefox/addon/cookies-txt/). Put exported file in the same directory as bot binary and name it `cookies.txt`.

### Local audio files
You can put audio files you want to play locally in `audio` directory.


## How to build
You need to have [Rust](https://www.rust-lang.org/tools/install) and [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed
1. Clone repository
2. Run `cargo build --release`
3. Built binary will be in `target/release` directory