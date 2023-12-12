# Reveal

Reveal is a simple, lightweight, and fast tool for League of Legends to reveal your team mates names in champ select. We also have dodging and a auto acceptor.

### Features

- Reveal your team mates names in champ select
- Auto acceptor
- Dodging & Last Second Dodging

### Usage

Head over to the [releases](https://github.com/steele123/reveal/releases) and download the precompiled binary, then simply just run it and it will automatically start revealing your team-mates names in champ select.

### Why this instead of LobbyReveal?

[LobbyReveal](https://github.com/Riotphobia/LobbyReveal) is great, but here's a few reasons.

- Smaller binary size (1.4MB)
- Less memory usage
- Not dependent on the .NET runtime (You don't need it installed)
- Faster startup time and faster execution time
- We don't spam the LCU API with requests, we only request the data as it's needed by connecting to the LCU's event websocket (This could make our tool a bit safer to use)
- GUI

### Safety

Most likely this tool is pretty safe, and you shouldn't get banned for using it, but I can't guarantee it. If you would like to increase the safety compiling it yourself won't hurt. If you get banned for using this tool, I'm not responsible.

![Screenshot](/tool.png)

## Credits

- [Shaco](https://github.com/Leastrio/Shaco/tree/main) for the LCU API wrapper so I didn't need to go through the pain.
- [LobbyReveal](https://github.com/Riotphobia/LobbyReveal) the original tool, thanks to [Inception](https://github.com/0xInception) for making it.
