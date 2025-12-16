#!/usr/bin/env python3
"""
Seed the forum database via API calls
"""
import requests
import json
import time

BASE_URL = "https://cyber-forum.onrender.com"

def clear_forum():
    """Clear all existing data"""
    print("Clearing existing forum data...")
    try:
        # Get all threads and delete them
        response = requests.get(f"{BASE_URL}/threads")
        if response.status_code == 200:
            threads = response.json()
            for thread in threads:
                # Delete all comments in thread
                comments_response = requests.get(f"{BASE_URL}/threads/{thread['id']}/comments")
                if comments_response.status_code == 200:
                    comments = comments_response.json()
                    for comment in comments:
                        requests.delete(f"{BASE_URL}/comments/{comment['id']}")
                # Delete thread
                requests.delete(f"{BASE_URL}/threads/{thread['id']}")
        print("Forum cleared!")
    except Exception as e:
        print(f"Error clearing forum: {e}")

def create_user(username):
    """Create a user"""
    try:
        response = requests.post(f"{BASE_URL}/register", json={"username": username})
        if response.status_code == 200:
            print(f"Created user: {username}")
            return response.json()
        else:
            # User might already exist
            print(f"User {username} might already exist")
            return {"id": "existing", "username": username}
    except Exception as e:
        print(f"Error creating user {username}: {e}")
        return None

def create_thread(title, content, username):
    """Create a thread"""
    try:
        response = requests.post(f"{BASE_URL}/threads", json={
            "title": title,
            "content": content,
            "author": username
        })
        if response.status_code == 200:
            thread = response.json()
            print(f"Created thread: {title}")
            return thread
        else:
            print(f"Error creating thread {title}: {response.text}")
            return None
    except Exception as e:
        print(f"Error creating thread {title}: {e}")
        return None

def create_comment(thread_id, content, username):
    """Create a comment"""
    try:
        response = requests.post(f"{BASE_URL}/comments", json={
            "thread_id": thread_id,
            "content": content,
            "author": username
        })
        if response.status_code == 200:
            print(f"Added comment to thread {thread_id}")
        else:
            print(f"Error adding comment: {response.text}")
    except Exception as e:
        print(f"Error adding comment: {e}")

def seed_forum():
    """Seed the forum with sample content"""
    print("Seeding forum with sample content...")
    
    # Create users
    users = ["arch_user", "linux_admin", "terminal_ninja", "rust_dev"]
    created_users = {}
    
    for username in users:
        user = create_user(username)
        if user:
            created_users[username] = user
        time.sleep(0.5)  # Rate limiting
    
    # Create threads
    threads_data = [
        ("Welcome to TERNIMAL!", "This is the official forum for the TERNIMAL terminal forum client. Feel free to discuss features, report bugs, or share your terminal setups!", "arch_user"),
        ("Best terminal emulators?", "What's your favorite terminal emulator? I've been using Alacritty lately but curious what others prefer.", "linux_admin"),
        ("Rust in terminal apps", "Building terminal apps with Rust is amazing! The performance and safety are unmatched. What terminal apps have you built?", "rust_dev"),
        ("Productivity tips", "Share your best terminal productivity tips! I'll start: tmux + vim + fzf is my holy trinity.", "terminal_ninja"),
        ("Arch vs other distros", "Why did you choose Arch Linux? Was it the AUR, the rolling release, or something else?", "arch_user"),
    ]
    
    created_threads = []
    for title, content, author in threads_data:
        thread = create_thread(title, content, author)
        if thread:
            created_threads.append((thread, author))
        time.sleep(0.5)  # Rate limiting
    
    # Add comments to specific threads
    if len(created_threads) >= 2:
        # Comments for "Best terminal emulators?"
        thread_id = created_threads[1][0]['id']
        create_comment(thread_id, "I'm still using gnome-terminal. It's simple and works well.", "arch_user")
        time.sleep(0.5)
        create_comment(thread_id, "Try Kitty! It's fast and has great GPU acceleration.", "terminal_ninja")
        time.sleep(0.5)
        create_comment(thread_id, "WezTerm is my favorite - cross platform and highly configurable.", "rust_dev")
    
    if len(created_threads) >= 3:
        # Comments for "Rust in terminal apps"
        thread_id = created_threads[2][0]['id']
        create_comment(thread_id, "I built a file manager in Rust! The compile times are worth it.", "rust_dev")
        time.sleep(0.5)
        create_comment(thread_id, "How's the binary size compared to C?", "linux_admin")
    
    print("\nForum seeded successfully!")
    print(f"Created {len(created_users)} users")
    print(f"Created {len(created_threads)} threads")
    print("Added sample comments")

if __name__ == "__main__":
    print("TERNIMAL Forum Seeding Script")
    print("=============================")
    
    # First clear existing data
    clear_forum()
    time.sleep(1)
    
    # Then seed with new content
    seed_forum()
    
    print("\nDone! You can now access the forum at:")
    print("TUI: cargo run")
    print("Web: https://cyber-forum.onrender.com/terminal")
