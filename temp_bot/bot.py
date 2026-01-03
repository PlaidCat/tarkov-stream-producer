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

if not TOKEN or not CHANNEL or 'YOUR_' in TOKEN or 'YOUR_' in CHANNEL:
    print("Error: Environment variables not set properly.")
    print("Please create a .env file based on .env.example and set TWITCH_TOKEN and TWITCH_CHANNEL.")
    sys.exit(1)

class Bot(commands.Bot):
    def __init__(self):
        # Prepare arguments for Bot initialization
        args = {
            'token': TOKEN,
            'prefix': '!',
            'initial_channels': [CHANNEL]
        }
        # Add optional client credentials if provided
        if CLIENT_ID:
            args['client_id'] = CLIENT_ID
        if CLIENT_SECRET:
            args['client_secret'] = CLIENT_SECRET
            
        super().__init__(**args)

    async def event_ready(self):
        print(f'Logged in as | {self.nick}')
        print(f'Connected to channel: {CHANNEL}')

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
    async def rules(self, ctx: commands.Context):
        response = self._read_response('rules.txt')
        await ctx.send(response)

    @commands.command()
    async def shopping(self, ctx: commands.Context):
        response = self._read_response('shopping.txt')
        await ctx.send(response)

    @commands.command()
    async def discord(self, ctx: commands.Context):
        await ctx.send("Join our Discord here: https://discord.gg/example")

    @commands.command()
    async def commands(self, ctx: commands.Context):
        await ctx.send("Available commands: !rules, !discord, !shopping")

if __name__ == "__main__":
    print("Starting bot...")
    try:
        bot = Bot()
        bot.run()
    except Exception as e:
        print(f"Error: {e}")
        print("Did you update the TOKEN and CHANNEL in bot.py?")
