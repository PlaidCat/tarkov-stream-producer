#!/usr/bin/env python3
"""
Helper script to fetch bot user ID from Twitch API.
Run this after setting up TWITCH_TOKEN and TWITCH_CLIENT_ID in .env
"""
import os
import sys
import requests
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

TOKEN = os.getenv('TWITCH_TOKEN')
CLIENT_ID = os.getenv('TWITCH_CLIENT_ID')

def validate_credentials():
    """Check if required credentials are set"""
    if not TOKEN or 'YOUR_' in TOKEN:
        print("Error: TWITCH_TOKEN not set in .env")
        print("Please set a valid OAuth token first.")
        return False

    if not CLIENT_ID or 'YOUR_' in CLIENT_ID:
        print("Error: TWITCH_CLIENT_ID not set in .env")
        print("Please set your client ID first.")
        return False

    return True

def get_bot_id():
    """Fetch bot user ID from Twitch Helix API"""
    # Remove 'oauth:' prefix if present
    token = TOKEN.replace('oauth:', '')

    headers = {
        'Authorization': f'Bearer {token}',
        'Client-Id': CLIENT_ID
    }

    try:
        response = requests.get(
            'https://api.twitch.tv/helix/users',
            headers=headers,
            timeout=10
        )

        if response.status_code == 401:
            print("Error: Invalid or expired OAuth token")
            print("Please generate a new token and update .env")
            return None

        if response.status_code != 200:
            print(f"Error: Twitch API returned status {response.status_code}")
            print(f"Response: {response.text}")
            return None

        data = response.json()

        if not data.get('data'):
            print("Error: No user data returned from Twitch API")
            return None

        user = data['data'][0]
        return {
            'id': user['id'],
            'login': user['login'],
            'display_name': user['display_name']
        }

    except requests.exceptions.RequestException as e:
        print(f"Error making API request: {e}")
        return None

if __name__ == "__main__":
    print("Twitch Bot ID Fetcher")
    print("=" * 50)

    if not validate_credentials():
        sys.exit(1)

    print("Fetching bot information from Twitch API...")

    bot_info = get_bot_id()

    if bot_info:
        print("\n✅ Success!")
        print(f"Bot Username: {bot_info['login']}")
        print(f"Display Name: {bot_info['display_name']}")
        print(f"Bot ID: {bot_info['id']}")
        print("\n" + "=" * 50)
        print("Add these lines to your .env file:")
        print(f"TWITCH_BOT_ID={bot_info['id']}")
        print(f"TWITCH_BOT_USERNAME={bot_info['login']}")
    else:
        print("\n❌ Failed to fetch bot ID")
        sys.exit(1)
