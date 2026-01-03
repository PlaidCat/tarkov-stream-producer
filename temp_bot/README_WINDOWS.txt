# Temporary Twitch Bot (Windows Setup)

This is a lightweight Python script to handle basic Twitch chat commands like !rules and !discord.

## Prerequisites

1.  **Python 3.7+**: Download and install from [python.org](https://www.python.org/downloads/).
    *   **IMPORTANT:** Check the box "Add Python to PATH" during installation.

## Configuration

1.  Create a new file named `.env` in this folder (or copy `.env.example` and rename it to `.env`).
2.  Open `.env` in a text editor.
3.  Get your **Access Token**:
    *   Go to [twitchtokengenerator.com](https://twitchtokengenerator.com/).
    *   Click "Bot Chat Token".
    *   Authorize with the Twitch account you want the bot to use.
    *   Copy the "Access Token" (e.g., `oauth:12345abcdef...`).
4.  Update `.env` with your details:
    *   Set `TWITCH_TOKEN` to your access token.
    *   Set `TWITCH_CHANNEL` to your Twitch channel name (lowercase).
    *   Save the file.

## Customizing Commands

The bot now reads responses from text files. You can edit these files while the bot is running (changes apply immediately):

*   **!rules**: Edits `rules.txt`.
*   **!shopping**: Edits `shopping.txt`.

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
