# Automated Story Generation System

This directory contains scripts and tools for generating original stories automatically.

## Files:

- `generate_story.py` - Main story generation script
- `README.md` - This documentation file

## Usage:

### Generate a story with command line:
```bash
python generate_story.py [theme] [length] [output_file]
```

### Examples:
```bash
# Generate a short fantasy story
python generate_story.py fantasy short fantasy_story.txt

# Generate a medium sci-fi story
python generate_story.py sci-fi medium scifi_story.txt

# Generate a long mystery story
python generate_story.py mystery long mystery_story.txt

# Generate and print to console
python generate_story.py contemporary short
```

## Available Themes:

- **fantasy** - Magical settings with wizards, dragons, and enchanted places
- **scifi** - Science fiction with space, technology, and futuristic concepts
- **mystery** - Detective stories with puzzles, clues, and hidden truths
- **contemporary** - Modern settings with everyday characters and situations

## Available Lengths:

- **short** - Flash fiction (200-300 words, ~8 sentences)
- **medium** - Short story (400-600 words, ~15 sentences)  
- **long** - Novella excerpt (800-1200 words, ~25 sentences)

## Features:

- **Randomized Content**: Each story uses different settings, characters, conflicts, and endings
- **Multiple Themes**: Supports fantasy, sci-fi, mystery, and contemporary genres
- **Flexible Lengths**: Three different story lengths for various needs
- **Metadata**: Includes generation timestamp and parameters
- **File Output**: Can save stories to files or print to console

## Story Structure:

Each generated story follows this format:
1. **Title** - Generated based on theme and content
2. **Setting** - The world/environment where the story takes place
3. **Character** - The protagonist and their unique trait
4. **Conflict** - The central challenge or problem they face
5. **Resolution** - How they overcome the challenge
6. **Metadata** - Generation information

## Customization:

To add new themes or modify existing ones, edit the `STORY_TEMPLATES` dictionary in `generate_story.py`. Each theme contains:
- `settings`: List of possible story locations
- `characters`: List of possible protagonists
- `conflicts`: List of possible central problems
- `endings`: List of possible resolutions

## Examples of Generated Stories:

The script creates original stories like:
- "The Mystic Apprentice's Lost Artifact and Learned that true magic comes from within"
- "Quantum Maintenance Worker's System Failure and discovered that hope can travel faster than light"
- "The Secret Detective's Generational Conspiracy and understood that every person has a story worth telling"

Each story is unique and combines different elements to create fresh narratives.