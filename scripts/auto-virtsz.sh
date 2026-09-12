#!/bin/bash

DIRECTORY="$1"

# Check if the directory argument was provided
if [ -z "$DIRECTORY" ]; then
  echo "Error: No directory path provided."
  echo "Usage: ./auto-virtsz.sh <path>"
  echo "path: dir of open pivots"
  exit 1
fi 

# Loop through all files (not subdirectories) in the specified directory
for file in "$DIRECTORY"/*; do
  # Check if the current item is a regular file
  if [ -f "$file" ]; then

    # 1. Create a safe temporary file
    temp_file=$(mktemp)

    # 2. Run your dapp/program and capture the output
    echo "processing file $file; saving to $temp_file"
    virtsz $LE_DATE $file > $temp_file

    # 3. Check the exit status of 'your_dapp_command'
    if [ $? -eq 0 ]; then
       echo "Processing succeeded! Updating file..."
       mv "$temp_file" "$file"
    else
       echo "Error: Dapp processing failed. Original file left untouched."

       # Clean up the temporary file
       rm -f "$temp_file"
    fi
  fi
done
