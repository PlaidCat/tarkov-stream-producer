# Temporary Twitch Bot (Windows Setup)

This is a lightweight Python script to handle basic Twitch chat commands like !rules and !discord.

## Prerequisites

1.  **Python 3.7+**: Download and install from [python.org](https://www.python.org/downloads/).
    *   **IMPORTANT:** Check the box "Add Python to PATH" during installation.

## Configuration

1.  Open `bot.py` in a text editor (Notepad, VS Code, etc.).
2.  Get your **Access Token**:
    *   Go to [twitchtokengenerator.com](https://twitchtokengenerator.com/).
    *   Click "Bot Chat Token".
    *   Authorize with the Twitch account you want the bot to use.
    *   Copy the "Access Token" (e.g., `oauth:12345abcdef...`).
3.  Update `bot.py`:
    *   Replace `'oauth:YOUR_ACCESS_TOKEN_HERE'` with your actual token.
    *   Replace `'YOUR_CHANNEL_NAME'` with your Twitch channel name (lowercase).
    *   Save the file.

## How to Run

1.  Double-click `run_bot.bat`.
2.  The script will automatically:
    *   Create a virtual environment (`venv`).
    *   Install the required library (`twitchio`).
    *   Start the bot.
3.  You should see "Logged in as | [YourBotName]" in the window.

## Adding Commands

To add more commands, edit `bot.py` and add new functions like this:

```python
    @commands.command()
    async def hello(self, ctx: commands.Context):
        await ctx.send(f"Hello there, {ctx.author.name}!")
```

Restart the bot (close the window and run `run_bot.bat` again) to apply changes.
