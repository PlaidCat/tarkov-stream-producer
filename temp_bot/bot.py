from twitchio.ext import commands

# CONFIGURATION
# TODO: Replace these with your actual details
# Get your token from: https://twitchtokengenerator.com/ (Select 'Bot Chat Token')
TOKEN = 'oauth:YOUR_ACCESS_TOKEN_HERE' # Keep the 'oauth:' prefix
CHANNEL = 'YOUR_CHANNEL_NAME'

class Bot(commands.Bot):
    def __init__(self):
        super().__init__(token=TOKEN, prefix='!', initial_channels=[CHANNEL])

    async def event_ready(self):
        print(f'Logged in as | {self.nick}')
        print(f'Connected to channel: {CHANNEL}')

    @commands.command()
    async def rules(self, ctx: commands.Context):
        await ctx.send("Be nice, no spam, and have fun!")

    @commands.command()
    async def discord(self, ctx: commands.Context):
        await ctx.send("Join our Discord here: https://discord.gg/example")

    @commands.command()
    async def commands(self, ctx: commands.Context):
        await ctx.send("Available commands: !rules, !discord")

if __name__ == "__main__":
    print("Starting bot...")
    try:
        bot = Bot()
        bot.run()
    except Exception as e:
        print(f"Error: {e}")
        print("Did you update the TOKEN and CHANNEL in bot.py?")
