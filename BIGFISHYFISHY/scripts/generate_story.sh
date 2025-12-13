#!/run/current-system/sw/bin/zsh
# Automated Story Generation Script (Bash Version)
# ================================================
#
# This script generates original stories based on predefined themes and templates.
# It can create stories with different genres, lengths, and styles.
#
# Usage:
#     ./generate_story.sh [theme] [length] [output_file]
#
# Example:
#     ./generate_story.sh fantasy short story_new.txt

# Story themes and components
declare -A FANTASY_SETTINGS
FANTASY_SETTINGS=(
    ["1"]="an enchanted forest where trees whispered ancient secrets"
    ["2"]="a floating city above the clouds"
    ["3"]="a library that existed between dimensions"
    ["4"]="a marketplace where magic was traded like currency"
    ["5"]="a castle built on the back of a sleeping dragon"
)

declare -A FANTASY_CHARACTERS
FANTASY_CHARACTERS=(
    ["1"]="a young apprentice with hidden powers"
    ["2"]="an elderly wizard who had forgotten his own spells"
    ["3"]="a thief who could steal memories"
    ["4"]="a princess who spoke to shadows"
    ["5"]="a blacksmith who forged weapons from starlight"
)

declare -A FANTASY_CONFLICTS
FANTASY_CONFLICTS=(
    ["1"]="must find a lost artifact before the moon turns red"
    ["2"]="discovers a prophecy that threatens everything they hold dear"
    ["3"]="must choose between saving their village or their true love"
    ["4"]="uncovers a conspiracy that goes to the heart of the kingdom"
    ["5"]="faces their greatest fear in a trial by elemental fire"
)

declare -A FANTASY_ENDINGS
FANTASY_ENDINGS=(
    ["1"]="and learned that true magic comes from within"
    ["2"]="and discovered that some treasures are worth more than gold"
    ["3"]="and found that courage grows stronger when shared"
    ["4"]="and realized that the greatest power is the power to choose"
    ["5"]="and understood that home is not a place but the people who love you"
)

# Generate random number
random_number() {
    echo $((1 + RANDOM % 5))
}

# Generate story
generate_story() {
    local theme=$1
    local length=$2
    local output_file=$3
    
    # Get random components
    local setting_num=$(random_number)
    local character_num=$(random_number)
    local conflict_num=$(random_number)
    local ending_num=$(random_number)
    
    # Get current date
    local current_date=$(date '+%Y-%m-%d %H:%M:%S')
    
    # Create story content
    local story_content="# The Mystic Story

In ${FANTASY_SETTINGS[$setting_num]}, there lived ${FANTASY_CHARACTERS[$character_num]}.

One day, they ${FANTASY_CONFLICTS[$conflict_num]}, ${FANTASY_ENDINGS[$ending_num]}.

*[Generated on $current_date using theme: $theme, length: $length]*"
    
    # Write to file or print
    if [ -n "$output_file" ]; then
        echo "$story_content" > "$output_file"
        echo "Story generated successfully: $output_file"
    else
        echo "$story_content"
    fi
}

# Main function
main() {
    if [ $# -eq 0 ]; then
        echo "Automated Story Generation Script"
        echo "Usage: $0 [theme] [length] [output_file]"
        echo "Available themes: fantasy"
        echo "Available lengths: short, medium, long"
        echo ""
        echo "Example: $0 fantasy short my_story.txt"
        return 1
    fi
    
    local theme=${1:-"fantasy"}
    local length=${2:-"short"}
    local output_file=$3
    
    generate_story "$theme" "$length" "$output_file"
}

# Run main function
main "$@"