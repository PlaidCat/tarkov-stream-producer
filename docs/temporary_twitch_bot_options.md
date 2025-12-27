# Temporary Twitch Bot Plan

**Objective:** Deploy a simple, free, open-source, self-hosted Twitch bot for basic commands (`!rules`, FAQ) while the custom Rust implementation is being built.

## Options Analysis

### 1. Firebot (Recommended for GUI)
*   **Type:** Desktop Application (Electron/Node.js)
*   **Repo:** [github.com/crowbartools/Firebot](https://github.com/crowbartools/Firebot)
*   **Pros:**
    *   Excellent Visual Interface (no coding needed).
    *   "Download and Run" simplicity.
    *   Powerful "Effects" system if you need more complexity later.
    *   Native Linux support (`.AppImage` or `.deb`).
*   **Cons:**
    *   Heavier resource usage (Electron) compared to a CLI script.

### 2. Custom Python Micro-Script (Recommended for Devs)
*   **Type:** Python Script (using `twitchio`)
*   **Pros:**
    *   Extremely lightweight (<50MB RAM).
    *   Instant setup (1 file).
    *   You have full control over the code.
    *   Easy to discard later.
*   **Cons:**
    *   Requires Python environment.
    *   Configuration via code/env vars.

### 3. OxidizeBot (Rust Alternative)
*   **Type:** Standalone Rust Application
*   **Repo:** [github.com/TwitchRecovery/OxidizeBot](https://github.com/TwitchRecovery/OxidizeBot)
*   **Pros:**
    *   Written in Rust (aligns with your project stack).
    *   High performance.
*   **Cons:**
    *   Setup can be slightly more involved than Firebot.

---

## The "5-Minute" Implementation Plan

We recommend **Option 2 (Python Script)** for the fastest, least intrusive setup, or **Option 1 (Firebot)** if you prefer a GUI.

### Path A: The Python Micro-Bot (Immediate Solution)

This script will give you a bot that responds to `!rules`, `!discord`, etc., in under 5 minutes.

**1. Prerequisites:**
   *   Python 3.7+ installed.
   *   A Twitch account for the bot (or use your main account).

**2. Get Credentials:**
   *   Go to [twitchtokengenerator.com](https://twitchtokengenerator.com/).
   *   Select "Bot Chat Token".
   *   Authorize and copy the **Access Token**.

**3. Setup:**
   ```bash
   # Create a folder
   mkdir temp_bot
   cd temp_bot

   # Create virtual env
   python3 -m venv venv
   source venv/bin/activate

   # Install twitchio
   pip install twitchio
   ```

**4. The Code (`bot.py`):**
   *(Copy this into `bot.py`)*
   ```python
   from twitchio.ext import commands

   # CONFIGURATION
   TOKEN = 'oauth:YOUR_ACCESS_TOKEN_HERE' # Keep the 'oauth:' prefix
   CHANNEL = 'YOUR_CHANNEL_NAME'

   class Bot(commands.Bot):
       def __init__(self):
           super().__init__(token=TOKEN, prefix='!', initial_channels=[CHANNEL])

       async def event_ready(self):
           print(f'Logged in as | {self.nick}')

       @commands.command()
       async def rules(self, ctx: commands.Context):
           await ctx.send("Be nice, no spam, and have fun!")

       @commands.command()
       async def discord(self, ctx: commands.Context):
           await ctx.send("Join our Discord here: https://discord.gg/example")

       @commands.command()
       async def commands(self, ctx: commands.Context):
           await ctx.send("Available commands: !rules, !discord")

   bot = Bot()
   bot.run()
   ```

**5. Run:**
   ```bash
   python bot.py
   ```

### Path B: Windows-Native Powerhouse Bots

If you are running on Windows, these are the two "gold standard" options for self-hosted bots. Both are highly visual and more powerful than a basic script.

#### 1. Mix It Up (Most User-Friendly)
*   **Type:** Windows Desktop App (.NET)
*   **Repo:** [github.com/MixItUpApp/MixItUp](https://github.com/MixItUpApp/MixItUp)
*   **Pros:**
    *   **Extremely intuitive:** Huge buttons, wizard-style setup.
    *   **Pre-built commands:** Comes with `!lurk`, `!discord`, etc., ready to go.
    *   **Open Source:** Very transparent and community-driven.
*   **Best for:** Someone who wants a "finished product" feel that just works.

#### 2. Streamer.bot (Performance & Automation)
*   **Type:** Windows Desktop App (Portable .exe)
*   **Website:** [streamer.bot](https://streamer.bot/)
*   **Pros:**
    *   **Lightweight:** Tiny executable, zero installation (portable).
    *   **Speed:** Insanely fast response times.
    *   **OBS Integration:** The best in the business for triggering OBS scene changes or sources via chat.
*   **Best for:** Developers or power users who want the "fastest" bot with future-proof automation features.

### Path C: Firebot (Cross-Platform GUI)

1.  **Download:** Go to the [Firebot Releases](https://github.com/crowbartools/Firebot/releases) and download the Linux `.AppImage` or Windows `.exe`.
2.  **Install/Run:** Execute the file.
3.  **Setup:**
    *   Click "Log in with Twitch".
    *   Go to "Commands" tab.
    *   Click "New Command".
    *   Trigger: `!rules`.
    *   Effect: "Chat Message" -> "Be nice!".
4.  **Active:** As long as Firebot is open, it will listen to chat.
