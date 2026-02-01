# Temporary Twitch Bot (Windows Setup)

This is a lightweight Python script to handle basic Twitch chat commands like !rules, !discord, and !shopping.

## Prerequisites

1.  **Python 3.7+**: Download and install from [python.org](https://www.python.org/downloads/).
    *   **IMPORTANT:** Check the box "Add Python to PATH" during installation.

2.  **Twitch Account**: You'll need a Twitch account for your bot (can be separate from your streaming account).

## Step 1: Register Your Application (Twitch Developer Console)

1.  Go to [https://dev.twitch.tv/console/apps](https://dev.twitch.tv/console/apps)
2.  Log in with your Twitch account
3.  Click **"Register Your Application"**
4.  Fill in the details:
    *   **Name**: Your bot name (e.g., "PlaidCatBot")
    *   **OAuth Redirect URLs**: `http://localhost:3000`
    *   **Category**: Chat Bot
5.  Click **"Create"**
6.  Click **"Manage"** on your new application
7.  **Copy your Client ID** - you'll need this
8.  Click **"New Secret"** and **copy the Client Secret immediately** (it only shows once!)

## Step 2: Get Your OAuth Token

**Option A: Using Twitch CLI (Recommended)**
1.  Download Twitch CLI from [https://github.com/twitchdev/twitch-cli/releases](https://github.com/twitchdev/twitch-cli/releases)
2.  Extract and open a command prompt in that folder
3.  Run: `twitch token -u -s "chat:read chat:edit"`
4.  Follow the browser prompts to authorize
5.  Copy the `Access Token` from the output

**Option B: Manual OAuth Flow**
1.  Replace YOUR_CLIENT_ID in this URL with your actual Client ID:
    ```
    https://id.twitch.tv/oauth2/authorize?client_id=YOUR_CLIENT_ID&redirect_uri=http://localhost:3000&response_type=token&scope=chat:read+chat:edit
    ```
2.  Visit the URL in your browser
3.  Authorize the application
4.  Copy the `access_token` from the redirect URL (after #access_token=)
5.  Add `oauth:` prefix to your token (e.g., `oauth:your_token_here`)

## Step 3: Configure Environment Variables

1.  Copy `.env.example` to `.env` (or create a new `.env` file)
2.  Open `.env` in Notepad or your favorite text editor
3.  Fill in these values:
    ```
    TWITCH_TOKEN=oauth:your_access_token_here
    TWITCH_CHANNEL=your_channel_name
    TWITCH_CLIENT_ID=your_client_id_here
    TWITCH_CLIENT_SECRET=your_client_secret_here
    TWITCH_BOT_ID=
    ```
4.  **Save the file** (leave TWITCH_BOT_ID empty for now)

## Step 4: Get Your Bot ID

1.  Open Command Prompt in this folder
2.  Run: `python get_bot_id.py`
3.  The script will display your bot's numeric user ID
4.  Copy the line it provides (e.g., `TWITCH_BOT_ID=123456789`)
5.  Open `.env` again and paste this line to complete your configuration
6.  **Save the file**

## Step 5: Run the Bot

1.  Double-click `run_bot.bat`
2.  The script will automatically:
    *   Create a virtual environment (`venv`)
    *   Install required libraries (twitchio, python-dotenv, requests, pytest)
    *   Start the bot
3.  You should see "Logged in as | [YourBotName]" in the window
4.  The bot is now running and will respond to commands in your Twitch chat!

## Available Commands

Once the bot is running, viewers can use these commands in your Twitch chat:

*   **!rules** - Shows your channel rules (edit `rules.txt` to customize)
*   **!shopping** - Shows your shopping list (edit `shopping.txt` to customize)
*   **!discord** - Shows Discord invite link (edit in `bot.py` line 67)
*   **!commands** - Lists all available commands

## Customizing Command Responses

You can edit these files while the bot is running (changes apply immediately):

*   **rules.txt** - Channel rules displayed by !rules command
*   **shopping.txt** - Shopping list displayed by !shopping command

No need to restart the bot when editing these files!

## Testing (Optional)

To run automated tests before going live:

1.  Open Command Prompt in this folder
2.  Run: `venv\Scripts\activate`
3.  Run: `pytest test_bot.py -v`
4.  All 12 tests should pass ✅

## Adding New Commands

To add more commands, edit `bot.py` and add new functions like this:

```python
@commands.command()
async def hello(self, ctx: commands.Context):
    await ctx.send(f"Hello there, {ctx.author.name}!")
```

Restart the bot (close the window and run `run_bot.bat` again) to apply code changes.

## Troubleshooting

**"Error: TWITCH_BOT_ID not set"**
- Run `python get_bot_id.py` to fetch your bot ID

**"Error: Invalid or expired OAuth token"**
- Your token may have expired. Generate a new one using Step 2

**"ModuleNotFoundError: No module named 'twitchio'"**
- Delete the `venv` folder and run `run_bot.bat` again

**Bot connects but doesn't respond to commands**
- Make sure you're typing commands in the correct Twitch channel
- Verify TWITCH_CHANNEL in .env matches your actual channel name (lowercase)

## Security Note

**NEVER share your `.env` file or commit it to version control!**
It contains sensitive credentials that could allow someone to control your bot.
