import os
import sys
from twitchio.ext import commands
from dotenv import load_dotenv

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
    print("Please create a .env file based on .env.example and set TWITCH_TOKEN and TWITCH_CHANNEL.")
    sys.exit(1)

# Validate bot_id is set
if not BOT_ID:
    print("Error: TWITCH_BOT_ID not set in .env")
    print("Run 'python get_bot_id.py' to fetch your bot ID.")
    sys.exit(1)

class Bot(commands.Bot):
    def __init__(self):
        # Prepare arguments for Bot initialization
        args = {
            'token': TOKEN,
            'prefix': '!',
            'initial_channels': [CHANNEL],
            'bot_id': BOT_ID
        }
        # Add optional client credentials if provided
        if CLIENT_ID:
            args['client_id'] = CLIENT_ID
        if CLIENT_SECRET:
            args['client_secret'] = CLIENT_SECRET
            
        super().__init__(**args)

    async def event_ready(self):
        print(f'Logged in as | {BOT_USERNAME}')
        print(f'Connected to channel: {CHANNEL}')

    async def event_command_error(self, context, error):
        """Handle command errors, including cooldowns"""
        if isinstance(error, commands.CommandOnCooldown):
            # Silently ignore cooldown errors to prevent chat spam
            # To notify users instead, uncomment: await context.send(f"⏱️ Command on cooldown ({error.retry_after:.0f}s)")
            pass
        else:
            # Log other errors
            print(f"Command error: {error}")

    def _read_response(self, filename):
        """Helper to read text from a file in the same directory."""
        try:
            # Construct absolute path to ensure we find the file
            base_path = os.path.dirname(os.path.abspath(__file__))
            file_path = os.path.join(base_path, filename)
            
            with open(file_path, 'r', encoding='utf-8') as f:
                return f.read().strip()
        except FileNotFoundError:
            return f"Error: {filename} not found."
        except Exception as e:
            print(f"Error reading {filename}: {e}")
            return "Error reading command file."

    @commands.command()
    @commands.cooldown(rate=1, per=30)
    async def rules(self, ctx: commands.Context):
        response = self._read_response('rules.txt')
        await ctx.send(response)

    @commands.command()
    @commands.cooldown(rate=1, per=30)
    async def shopping(self, ctx: commands.Context):
        response = self._read_response('shopping.txt')
        await ctx.send(response)

    @commands.command()
    @commands.cooldown(rate=1, per=30)
    async def tarkov_pve(self, ctx: commands.Context):
        response = self._read_response('tarkov_pve.txt')
        await ctx.send(response)

    @commands.command()
    @commands.cooldown(rate=1, per=30)
    async def commands(self, ctx: commands.Context):
        await ctx.send("Available commands: !rules, !shopping, !tarkov_pve")

if __name__ == "__main__":
    print("Starting bot...")
    try:
        bot = Bot()
        bot.run()
    except Exception as e:
        print(f"Error: {e}")
        print("Did you update the TOKEN and CHANNEL in bot.py?")
