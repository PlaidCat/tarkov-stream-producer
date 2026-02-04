import os
import sys
import logging
import asyncio
from twitchio import eventsub
from twitchio.ext import commands
from dotenv import load_dotenv

# Configure Logging
logging.basicConfig(level=logging.INFO, format='%(name)s - %(levelname)s - %(message)s')
logging.getLogger("twitchio.websocket").setLevel(logging.DEBUG)

# Load environment variables from .env file
load_dotenv()

# CONFIGURATION
TOKEN = os.getenv('TWITCH_TOKEN')
CHANNEL = os.getenv('TWITCH_CHANNEL')
CLIENT_ID = os.getenv('TWITCH_CLIENT_ID')
CLIENT_SECRET = os.getenv('TWITCH_CLIENT_SECRET')
BOT_ID = os.getenv('TWITCH_BOT_ID')
BOT_USERNAME = os.getenv('TWITCH_BOT_USERNAME', 'Bot')

if not TOKEN or not CHANNEL or 'YOUR_' in TOKEN or 'YOUR_' in CHANNEL:
    print("Error: Environment variables not set properly.")
    sys.exit(1)

if not CLIENT_ID or not CLIENT_SECRET or not BOT_ID:
    print("Error: TWITCH_CLIENT_ID, TWITCH_CLIENT_SECRET, and TWITCH_BOT_ID must be set.")
    sys.exit(1)

# Normalize configuration
if TOKEN.startswith('oauth:'):
    TOKEN = TOKEN.replace('oauth:', '')

CHANNEL = CHANNEL.lower()

class BotCommands(commands.Component):
    def __init__(self, bot: commands.Bot):
        self.bot = bot

    def _read_response(self, filename):
        try:
            base_path = os.path.dirname(os.path.abspath(__file__))
            file_path = os.path.join(base_path, filename)
            with open(file_path, 'r', encoding='utf-8') as f:
                return f.read().strip()
        except Exception as e:
            print(f"Error reading {filename}: {e}")
            return "Error reading command file."

    @commands.command()
    async def rules(self, ctx: commands.Context):
        response = self._read_response('rules.txt')
        await ctx.reply(response)

    @commands.command()
    async def shopping(self, ctx: commands.Context):
        response = self._read_response('shopping.txt')
        await ctx.reply(response)

    @commands.command()
    async def tarkov_pve(self, ctx: commands.Context):
        response = self._read_response('tarkov_pve.txt')
        await ctx.reply(response)

    @commands.command(name="commands")
    async def command_list(self, ctx: commands.Context):
        # Renamed to command_list to avoid conflict with module name, but command name is "commands"
        await ctx.reply("Available commands: !rules, !shopping, !tarkov_pve")

class Bot(commands.Bot):
    def __init__(self):
        super().__init__(
            client_id=CLIENT_ID,
            client_secret=CLIENT_SECRET,
            bot_id=BOT_ID,
            prefix='!'
        )

    async def setup_hook(self):
        # In twitchio 3.0+, we must subscribe to events explicitly
        print(f"Setting up EventSub subscriptions for channel: {CHANNEL}")
        try:
            # Fetch the broadcaster user object to get their ID
            users = await self.fetch_users(logins=[CHANNEL])
            if not users:
                print(f"Error: Could not find user {CHANNEL}")
                return
            
            broadcaster = users[0]
            print(f"Found broadcaster {broadcaster.name} (ID: {broadcaster.id})")
            
            # Subscribe to chat messages
            payload = eventsub.ChatMessageSubscription(
                broadcaster_user_id=broadcaster.id,
                user_id=self.bot_id
            )
            await self.subscribe_websocket(payload=payload)
            print("Successfully subscribed to ChatMessage events!")
            
            # Add commands component
            await self.add_component(BotCommands(self))
            print("Successfully added BotCommands component!")
            
        except Exception as e:
            print(f"Failed in setup_hook: {e}")
            import traceback
            traceback.print_exc()

    async def event_ready(self):
        print(f'Logged in as | {BOT_USERNAME} (ID: {BOT_ID})')
        print(f'Connected to Twitch. Listening for commands in: {CHANNEL}')

    async def event_join(self, channel, user):
        print(f'Event Join: User {user.name} joined {channel.name}')

    async def event_error(self, payload):
        """Handle errors in twitchio 3.0+"""
        print(f"ERROR in bot listener: {payload.error}")
        if payload.original:
            print(f"Original payload: {payload.original}")

    async def event_message(self, message):
        """Handle incoming messages via EventSub"""
        print(f"RAW MESSAGE RECEIVED: {message.text}")
        
        # In 3.0, echo is handled differently or we check the chatter ID
        # Commenting this out because you are testing with the bot account!
        # if message.chatter.id == self.bot_id:
        #    return

        print(f"[DEBUG] Message from {message.chatter.name}: {message.text}")
        await self.process_commands(message)

async def main():
    try:
        async with Bot() as bot:
            print(f"Adding user token for bot ID: {BOT_ID}...")
            # In TwitchIO 3.0, add_token registers the token for the user.
            # We use an empty string for refresh since we don't have one.
            await bot.add_token(TOKEN, "")
            
            print("Starting bot...")
            # start() without token will use the app token or the managed tokens.
            await bot.start(load_tokens=False)
    except KeyboardInterrupt:
        pass
    except Exception as e:
        print(f"Fatal error in main: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    print("Starting bot (TwitchIO 3.0+)...")
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
