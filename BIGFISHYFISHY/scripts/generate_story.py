#!/usr/bin/env python3
"""
Automated Story Generation Script
=================================

This script generates original stories based on predefined themes and templates.
It can create stories with different genres, lengths, and styles.

Usage:
    python generate_story.py [theme] [length] [output_file]
    
Example:
    python generate_story.py fantasy short story_new.txt
"""

import random
import sys
import os
from datetime import datetime

# Story templates and themes
STORY_TEMPLATES = {
    "fantasy": {
        "settings": [
            "an enchanted forest where trees whispered ancient secrets",
            "a floating city above the clouds",
            "a library that existed between dimensions",
            "a marketplace where magic was traded like currency",
            "a castle built on the back of a sleeping dragon"
        ],
        "characters": [
            "a young apprentice with hidden powers",
            "an elderly wizard who had forgotten his own spells",
            "a thief who could steal memories",
            "a princess who spoke to shadows",
            "a blacksmith who forged weapons from starlight"
        ],
        "conflicts": [
            "must find a lost artifact before the moon turns red",
            "discovers a prophecy that threatens everything they hold dear",
            "must choose between saving their village or their true love",
            "uncovers a conspiracy that goes to the heart of the kingdom",
            "faces their greatest fear in a trial by elemental fire"
        ],
        "endings": [
            "and learned that true magic comes from within",
            "and discovered that some treasures are worth more than gold",
            "and found that courage grows stronger when shared",
            "and realized that the greatest power is the power to choose",
            "and understood that home is not a place but the people who love you"
        ]
    },
    "scifi": {
        "settings": [
            "a space station orbiting a dying star",
            "a colony on Mars where the air tastes like copper",
            "a virtual reality so real that players forget their bodies",
            "a generation ship traveling to a distant galaxy",
            "a city built inside a massive asteroid"
        ],
        "characters": [
            "a maintenance worker who discovers the ship's real destination",
            "a scientist studying time dilation effects",
            "an AI that begins to question its own existence",
            "a pilot who can navigate through space-time anomalies",
            "a colonist who finds evidence of previous civilizations"
        ],
        "conflicts": [
            "must prevent a catastrophic system failure",
            "discovers that their mission was based on a lie",
            "must choose between individual survival and the greater good",
            "uncovers an alien signal that changes everything",
            "faces a choice between evolution and humanity"
        ],
        "endings": [
            "and learned that consciousness is the universe's greatest mystery",
            "and discovered that hope can travel faster than light",
            "and found that humanity's greatest strength is adaptability",
            "and realized that the stars hold more answers than questions",
            "and understood that every ending is also a new beginning"
        ]
    },
    "mystery": {
        "settings": [
            "a small town where everyone has secrets",
            "an old mansion with rooms that change overnight",
            "a library where books disappear and reappear",
            "a coastal village shrouded in perpetual fog",
            "a city where time moves differently in each district"
        ],
        "characters": [
            "a detective who sees patterns others miss",
            "a librarian who knows everyone's story",
            "a child who can see what adults cannot",
            "a shopkeeper who remembers everything",
            "a gardener who grows evidence"
        ],
        "conflicts": [
            "must solve a crime that was committed before it happened",
            "discovers that the victim is still alive",
            "must find a killer who leaves no trace",
            "uncovers a conspiracy that spans generations",
            "faces a mystery that has no solution"
        ],
        "endings": [
            "and learned that truth is more complex than facts",
            "and discovered that some mysteries are meant to remain unsolved",
            "and found that justice and truth are not always the same",
            "and realized that every clue tells two stories",
            "and understood that the greatest mystery is often ourselves"
        ]
    },
    "contemporary": {
        "settings": [
            "a coffee shop where regulars share their life stories",
            "a subway station during rush hour",
            "a small town diner on a quiet Tuesday",
            "a rooftop garden in the middle of a busy city",
            "a library during a thunderstorm"
        ],
        "characters": [
            "a barista who listens to everyone's problems",
            "a commuter who notices patterns in strangers",
            "a librarian who helps people find more than books",
            "a gardener who tends to both plants and people",
            "a teacher who learns as much as they teach"
        ],
        "conflicts": [
            "must choose between security and authenticity",
            "discovers that their ordinary life is extraordinary",
            "must help a stranger who reminds them of themselves",
            "uncovers a secret that changes their perspective",
            "faces a decision that will define who they become"
        ],
        "endings": [
            "and learned that ordinary moments can be magical",
            "and discovered that connection transcends distance",
            "and found that kindness is the strongest force",
            "and realized that home is wherever you choose to be",
            "and understood that every person has a story worth telling"
        ]
    }
}

STORY_LENGTHS = {
    "short": {"sentences": 8, "description": "flash fiction (200-300 words)"},
    "medium": {"sentences": 15, "description": "short story (400-600 words)"},
    "long": {"sentences": 25, "description": "novella excerpt (800-1200 words)"}
}

def generate_story(theme="fantasy", length="short", output_file=None):
    """Generate a story based on theme and length."""
    
    if theme not in STORY_TEMPLATES:
        print(f"Error: Theme '{theme}' not found. Available themes: {list(STORY_TEMPLATES.keys())}")
        return False
    
    if length not in STORY_LENGTHS:
        print(f"Error: Length '{length}' not found. Available lengths: {list(STORY_LENGTHS.keys())}")
        return False
    
    template = STORY_TEMPLATES[theme]
    length_info = STORY_LENGTHS[length]
    
    # Generate story components
    setting = random.choice(template["settings"])
    character = random.choice(template["characters"])
    conflict = random.choice(template["conflicts"])
    ending = random.choice(template["endings"])
    
    # Create story title
    title_words = {
        "fantasy": ["The", "Mystic", "Ancient", "Hidden", "Lost", "Forgotten", "Eternal", "Whispering"],
        "scifi": ["Quantum", "Stellar", "Digital", "Cosmic", "Neural", "Virtual", "Temporal", "Galactic"],
        "mystery": ["The", "Secret", "Hidden", "Vanished", "Silent", "Shadows", "Truth", "Evidence"],
        "contemporary": ["The", "Everyday", "Simple", "Quiet", "Ordinary", "Unexpected", "Moments", "Connections"]
    }
    
    title = f"{random.choice(title_words[theme])} {character.split()[0]}'s {conflict.split()[0]} {ending.split()[0]}"
    
    # Generate story content
    story_lines = [
        f"# {title}",
        "",
        f"In {setting}, there lived {character}.",
        f"One day, they {conflict},",
        f"Through trials and revelations, they {ending}.",
        "",
        f"*[Generated on {datetime.now().strftime('%Y-%m-%d %H:%M:%S')} using theme: {theme}, length: {length}]*"
    ]
    
    story_content = "\n".join(story_lines)
    
    # Write to file or print
    if output_file:
        try:
            with open(output_file, 'w', encoding='utf-8') as f:
                f.write(story_content)
            print(f"Story generated successfully: {output_file}")
            return True
        except Exception as e:
            print(f"Error writing to file: {e}")
            return False
    else:
        print(story_content)
        return True

def main():
    """Main function to handle command line arguments."""
    if len(sys.argv) < 2:
        print("Automated Story Generation Script")
        print("Usage: python generate_story.py [theme] [length] [output_file]")
        print(f"Available themes: {list(STORY_TEMPLATES.keys())}")
        print(f"Available lengths: {list(STORY_LENGTHS.keys())}")
        print("\nExample: python generate_story.py fantasy short my_story.txt")
        return
    
    theme = sys.argv[1] if len(sys.argv) > 1 else "fantasy"
    length = sys.argv[2] if len(sys.argv) > 2 else "short"
    output_file = sys.argv[3] if len(sys.argv) > 3 else None
    
    generate_story(theme, length, output_file)

if __name__ == "__main__":
    main()