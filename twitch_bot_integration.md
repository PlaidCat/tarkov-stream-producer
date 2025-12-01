# Twitch Bot Integration - Technical Details

This document provides deep integration details for the Twitch chat bot component of Tarkov Stream Producer.

## Library Choice: twitch-irc

**Repository:** https://github.com/robotty/twitch-irc-rs
**Version:** 5.0+
**License:** MIT/Apache-2.0

### Why twitch-irc?
- Most mature and actively maintained Rust Twitch library
- Async/await native (works with tokio)
- Handles IRC protocol details automatically
- Supports both anonymous and authenticated connections
- Good error handling and reconnection logic

## Authentication Options

### Option 1: Anonymous (Read-Only) ✅ RECOMMENDED FOR PHASE 2
```rust
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::ClientConfig;

let config = ClientConfig::default();
let (mut incoming_messages, client) =
    TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);
```

**Pros:**
- No OAuth token required
- Can read all chat messages
- Perfect for responding to commands via external API

**Cons:**
- Cannot send messages to chat
- Read-only mode

### Option 2: Authenticated (Read/Write)
```rust
use twitch_irc::login::StaticLoginCredentials;

let credentials = StaticLoginCredentials::new(
    "your_bot_username".to_string(),
    Some("oauth:your_token_here".to_string())
);
let config = ClientConfig::new_simple(credentials);
```

**When to use:**
- Phase 3+ when bot needs to reply in chat
- Requires Twitch OAuth token (get from https://twitchtokengenerator.com/)

## Bot Commands Design

### !stats - Overall Statistics
**Response format:** "Session stats: 5 raids, 12 kills, 3 survived (60% SR), 2.4 K/D"

**Data needed:**
- Total raids in current session
- Total kills in current session
- Raids survived count
- Survival rate calculation
- K/D ratio calculation

**Database query:**
```sql
SELECT
    COUNT(*) as total_raids,
    SUM(kills) as total_kills,
    SUM(CASE WHEN survived = 1 THEN 1 ELSE 0 END) as survived_count
FROM raids
WHERE started_at >= [session_start_time]
```

### !kd - Kill/Death Ratio
**Response format:** "K/D: 2.4 (12 kills, 5 deaths)"

**Data needed:**
- Total kills
- Total deaths (raids where survived = false)

**Database query:**
```sql
SELECT
    SUM(kills) as total_kills,
    COUNT(*) - SUM(CASE WHEN survived = 1 THEN 1 ELSE 0 END) as total_deaths
FROM raids
WHERE started_at >= [session_start_time]
```

### !raid - Current Raid Info
**Response format:** "Current raid: Customs, 3 kills, 12m elapsed"

**Data needed:**
- Current raid map
- Current raid kills
- Raid start time (calculate elapsed)

**Database query:**
```sql
SELECT map, kills, started_at
FROM raids
WHERE ended_at IS NULL
ORDER BY started_at DESC
LIMIT 1
```

### !session - Session Summary
**Response format:** "Today's session: 5 raids, 2h 15m, avg 3.2 kills/raid"

**Data needed:**
- Total session time
- Raid count
- Average kills per raid

## Rate Limiting

Twitch chat has rate limits:
- **Unverified bots:** 20 messages per 30 seconds
- **Verified bots:** 100 messages per 30 seconds

**Implementation strategy:**
- Use a simple cooldown per command (e.g., 5 seconds)
- Track last command execution time in memory
- Return early if cooldown hasn't expired

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CommandCooldown {
    last_executed: HashMap<String, Instant>,
    cooldown_duration: Duration,
}

impl CommandCooldown {
    fn can_execute(&mut self, command: &str) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_executed.get(command) {
            if now.duration_since(*last) < self.cooldown_duration {
                return false;
            }
        }
        self.last_executed.insert(command.to_string(), now);
        true
    }
}
```

## Configuration

Store bot configuration in environment variables or a config file:

```toml
# config.toml
[twitch]
channel = "your_channel_name"
bot_username = "your_bot_username"  # Optional, for authenticated mode
oauth_token = "oauth:token_here"     # Optional, for authenticated mode
command_cooldown_seconds = 5
```

Or use environment variables:
```bash
TWITCH_CHANNEL=your_channel_name
TWITCH_BOT_USERNAME=bot_name
TWITCH_OAUTH_TOKEN=oauth:token
```

## Error Handling

Common errors to handle:
- **Connection failures:** Retry with exponential backoff
- **Invalid channel:** Validate channel exists before joining
- **Rate limit exceeded:** Implement cooldowns proactively
- **Database errors:** Return graceful error message instead of crashing

## Testing Strategy

### Unit Tests
- Test command parsing logic
- Test cooldown mechanism
- Test data formatting functions

### Integration Tests
- Use mock IRC server (or test channel)
- Verify commands trigger correct database queries
- Test error scenarios

### Manual Testing
- Connect to your actual Twitch channel
- Test each command with real data
- Verify rate limiting works
- Check reconnection logic

## Phase 2 Implementation Checklist

- [ ] Add twitch-irc dependency
- [ ] Create bot module with connection logic
- [ ] Implement anonymous authentication
- [ ] Add command detection (!stats, !kd, !raid)
- [ ] Create placeholder responses (mock data)
- [ ] Add basic error handling
- [ ] Write unit tests for command parsing
- [ ] Test connection to real Twitch channel

## Phase 3 Extensions (Future)

- [ ] Switch to authenticated mode
- [ ] Implement actual chat responses
- [ ] Add database integration for real data
- [ ] Implement command cooldowns
- [ ] Add logging for all commands
- [ ] Create admin commands (!reset, !clear)
- [ ] Add per-user cooldowns (not just global)

## Resources

- [twitch-irc docs](https://docs.rs/twitch-irc/)
- [Twitch IRC guide](https://dev.twitch.tv/docs/irc/)
- [Twitch token generator](https://twitchtokengenerator.com/)
- [Twitch rate limits](https://dev.twitch.tv/docs/irc/#rate-limits)
