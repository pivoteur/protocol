#!/bin/bash

# Assign the first argument to a variable named 'target_dir'
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
    # *** Replace the following line with your specific command ***
    echo "processing file $file"
    virtsz pivot $LE_DATE $file > $file
    # Example: run a data processing command on the file
    # virtsz <protocol> <date> <path or "$file">
  fi
done
