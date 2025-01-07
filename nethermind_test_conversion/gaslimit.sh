#!/bin/bash
# 
# This file converts Nethermind .txt files into json files that this library can process.
# Scroll down to show_usage to see what arguments this script takes
# 
# Function to process a single file
process_file() {
    local input_file="$1"
    local output_dir="$2"
    local base_name=$(basename "$input_file")
    local output_file="$output_dir/${base_name%.*}.json"

    # Extract gas limit from filename
    local gas_limit=$(basename "$input_file" | grep -o '[0-9]\+M' | sed 's/M$//')
    if [ -z "$gas_limit" ]; then
        echo "Skipping $input_file: Filename must contain a gas limit in the format 'GasLimit_XXM.txt'"
        return 1
    fi

    # Create output directory if it doesn't exist
    mkdir -p "$(dirname "$output_file")"

    # Create JSON structure
    cat > "$output_file" << EOF
{
    "name": "gas_limit_${gas_limit}M",
    "description": "Calls a block which uses ${gas_limit}M Gas",
    "sequence": [
EOF

    # Process the input file
    local current_request=""
    local sequence_index=0
    local descriptions=(
        "Empty block with withdrawals"
        "Fork Choice update"
        "Contract deployment"
        "Fork Choice update"
        "Call contract and use up ${gas_limit}M GAS"
        "Fork Choice update"
    )
    local expect_measurements=(
        false
        false
        false
        false
        true
        false
    )

    while IFS= read -r line || [ -n "$line" ]; do
        # Skip empty lines
        if [ -z "$line" ]; then
            continue
        fi

        # If we have a complete JSON request
        if [ -n "$current_request" ] && echo "$line" | grep -q '^{"jsonrpc":'; then
            # Write previous request
            if [ $sequence_index -gt 0 ]; then
                echo "        }," >> "$output_file"
            fi
            
            # Start new sequence item
            cat >> "$output_file" << EOF
        {
            "description": "${descriptions[$sequence_index]}",
            "expect_measurement": ${expect_measurements[$sequence_index]},
            "request": $current_request
EOF
            sequence_index=$((sequence_index + 1))
            current_request="$line"
        else
            # Append to current request
            if [ -n "$current_request" ]; then
                current_request="$current_request"$'\n'"$line"
            else
                current_request="$line"
            fi
        fi
    done < "$input_file"

    # Write the last request
    if [ -n "$current_request" ]; then
        if [ $sequence_index -gt 0 ]; then
            echo "        }," >> "$output_file"
        fi
        cat >> "$output_file" << EOF
        {
            "description": "${descriptions[$sequence_index]}",
            "expect_measurement": ${expect_measurements[$sequence_index]},
            "request": $current_request
        }
EOF
    fi

    # Close the JSON structure
    cat >> "$output_file" << EOF
    ]
}
EOF

    # Format the JSON (requires jq)
    if command -v jq >/dev/null 2>&1; then
        local temp_file="${output_file}.tmp"
        jq . "$output_file" > "$temp_file" && mv "$temp_file" "$output_file"
        echo "✓ Converted and formatted $input_file to $output_file with gas limit ${gas_limit}M"
    else
        echo "✓ Converted $input_file to $output_file with gas limit ${gas_limit}M (install jq for proper JSON formatting)"
    fi
}

# Show usage
show_usage() {
    echo "Usage: $0 <input_path> [output_directory]"
    echo "  input_path: File or directory containing GasLimit_XXM.txt files"
    echo "  output_directory: Optional. Directory where JSON files will be created"
    echo "                    (defaults to same directory as input)"
}

# Main script
if [ $# -lt 1 ]; then
    show_usage
    exit 1
fi

input_path="$1"
output_dir="${2:-$(dirname "$input_path")}"

# Ensure output directory exists
mkdir -p "$output_dir"

if [ -d "$input_path" ]; then
    # Process directory
    echo "Processing directory: $input_path"
    echo "Output directory: $output_dir"
    echo "-------------------"
    
    # Find all txt files in the directory
    find "$input_path" -type f -name "GasLimit_*M.txt" | while read -r file; do
        process_file "$file" "$output_dir"
    done
    
    echo "-------------------"
    echo "Processing complete!"
elif [ -f "$input_path" ]; then
    # Process single file
    process_file "$input_path" "$output_dir"
else
    echo "Error: '$input_path' is not a valid file or directory"
    exit 1
fi