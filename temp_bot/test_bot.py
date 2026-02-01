"""
Tests for Twitch bot commands and functionality.
Run with: pytest test_bot.py -v
"""
import pytest
import os
import sys
from unittest.mock import Mock, AsyncMock, patch, mock_open
from pathlib import Path

# Add parent directory to path to import bot
sys.path.insert(0, str(Path(__file__).parent))

# Mock environment variables before importing bot
@pytest.fixture(autouse=True)
def mock_env():
    """Mock all required environment variables"""
    with patch.dict(os.environ, {
        'TWITCH_TOKEN': 'oauth:test_token_12345',
        'TWITCH_CHANNEL': 'test_channel',
        'TWITCH_CLIENT_ID': 'test_client_id',
        'TWITCH_CLIENT_SECRET': 'test_client_secret',
        'TWITCH_BOT_ID': '123456789',
        'TWITCH_BOT_USERNAME': 'test_bot'
    }):
        yield

@pytest.fixture
def bot_instance():
    """Create a bot instance for testing"""
    # Import here to use mocked env vars
    from bot import Bot
    return Bot()

@pytest.fixture
def mock_context():
    """Create a mock TwitchIO context"""
    ctx = Mock()
    ctx.send = AsyncMock()
    return ctx

class TestBotInitialization:
    """Test bot initialization and validation"""

    def test_bot_creates_successfully(self, bot_instance):
        """Bot should initialize with valid credentials"""
        assert bot_instance is not None

    def test_missing_token_fails(self):
        """Bot should exit if TOKEN is missing"""
        with patch.dict(os.environ, {'TWITCH_TOKEN': ''}, clear=True):
            with pytest.raises(SystemExit):
                # Re-import to trigger validation
                import importlib
                import bot as bot_module
                importlib.reload(bot_module)

    def test_missing_bot_id_fails(self):
        """Bot should exit if BOT_ID is missing"""
        env = {
            'TWITCH_TOKEN': 'oauth:test',
            'TWITCH_CHANNEL': 'test',
            'TWITCH_BOT_ID': ''
        }
        with patch.dict(os.environ, env, clear=True):
            with pytest.raises(SystemExit):
                import importlib
                import bot as bot_module
                importlib.reload(bot_module)

class TestFileReading:
    """Test the _read_response helper method"""

    def test_read_existing_file(self, bot_instance, tmp_path):
        """Should read and return file contents"""
        # Create a temporary test file
        test_file = tmp_path / "test.txt"
        test_file.write_text("Test content here")

        # Mock the file path resolution
        with patch('os.path.dirname', return_value=str(tmp_path)):
            result = bot_instance._read_response("test.txt")
            assert result == "Test content here"

    def test_read_missing_file(self, bot_instance):
        """Should return error message for missing file"""
        result = bot_instance._read_response("nonexistent.txt")
        assert "Error:" in result
        assert "not found" in result

    def test_read_file_strips_whitespace(self, bot_instance, tmp_path):
        """Should strip leading/trailing whitespace"""
        test_file = tmp_path / "whitespace.txt"
        test_file.write_text("  \n  Content with spaces  \n  ")

        with patch('os.path.dirname', return_value=str(tmp_path)):
            result = bot_instance._read_response("whitespace.txt")
            assert result == "Content with spaces"

class TestCommands:
    """Test bot command responses"""

    @pytest.mark.asyncio
    async def test_rules_command(self, bot_instance, mock_context, tmp_path):
        """!rules should send rules.txt content"""
        # Create mock rules file
        rules_file = tmp_path / "rules.txt"
        rules_file.write_text("1. Be nice\n2. Have fun")

        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.rules.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "Be nice" in call_args
        assert "Have fun" in call_args

    @pytest.mark.asyncio
    async def test_shopping_command(self, bot_instance, mock_context, tmp_path):
        """!shopping should send shopping.txt content"""
        shopping_file = tmp_path / "shopping.txt"
        shopping_file.write_text("Shopping List:\n- Item 1\n- Item 2")

        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.shopping.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "Shopping List" in call_args
        assert "Item 1" in call_args

    @pytest.mark.asyncio
    async def test_tarkov_pve_command(self, bot_instance, mock_context, tmp_path):
        """!tarkov_pve should send tarkov_pve.txt content"""
        pve_file = tmp_path / "tarkov_pve.txt"
        pve_file.write_text("PVE info here")

        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.tarkov_pve.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "PVE info" in call_args

    @pytest.mark.asyncio
    async def test_commands_command(self, bot_instance, mock_context):
        """!commands should list available commands"""
        await bot_instance.commands.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "!rules" in call_args
        assert "!shopping" in call_args
        assert "!tarkov_pve" in call_args

class TestErrorHandling:
    """Test error handling scenarios"""

    @pytest.mark.asyncio
    async def test_rules_file_missing(self, bot_instance, mock_context, tmp_path):
        """Should send error message if rules.txt missing"""
        # Point to empty directory where file doesn't exist
        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.rules.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "Error" in call_args

    @pytest.mark.asyncio
    async def test_shopping_file_missing(self, bot_instance, mock_context, tmp_path):
        """Should send error message if shopping.txt missing"""
        # Point to empty directory where file doesn't exist
        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.shopping.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "Error" in call_args

    @pytest.mark.asyncio
    async def test_tarkov_pve_file_missing(self, bot_instance, mock_context, tmp_path):
        """Should send error message if tarkov_pve.txt missing"""
        # Point to empty directory where file doesn't exist
        with patch('os.path.dirname', return_value=str(tmp_path)):
            await bot_instance.tarkov_pve.callback(bot_instance, mock_context)

        mock_context.send.assert_called_once()
        call_args = mock_context.send.call_args[0][0]
        assert "Error" in call_args

class TestCooldowns:
    """Test command cooldown functionality"""

    @pytest.mark.asyncio
    async def test_commands_have_cooldowns(self, bot_instance):
        """Verify all commands have cooldown decorators"""
        # Import commands module to access Cooldown
        from twitchio.ext import commands as twitchio_commands

        # Check that each command has a cooldown
        assert hasattr(bot_instance.rules, '_buckets')
        assert hasattr(bot_instance.shopping, '_buckets')
        assert hasattr(bot_instance.tarkov_pve, '_buckets')
        assert hasattr(bot_instance.commands, '_buckets')

if __name__ == "__main__":
    pytest.main([__file__, "-v"])
