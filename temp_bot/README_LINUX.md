# Temporary Twitch Bot (Linux Setup)

## Prerequisites

1.  **Python 3**: Usually installed by default. If not:
    ```bash
    sudo apt update
    sudo apt install python3 python3-venv python3-pip
    ```

## Configuration

1.  **Create .env file**:
    ```bash
    cp .env.example .env
    ```
2.  **Edit .env**:
    *   Open `.env` in your text editor (nano, vim, gedit, etc.).
    *   Set `TWITCH_TOKEN` (from https://twitchtokengenerator.com/).
    *   Set `TWITCH_CHANNEL`.
    *   Save the file.

## How to Run

1.  **Open a terminal** in this folder.
2.  **Run the script**:
    ```bash
    ./run_bot.sh
    ```

The script will automatically handle the virtual environment and dependencies.

## Customizing Commands

You can edit `rules.txt` and `shopping.txt` to change the bot's responses without restarting it.
