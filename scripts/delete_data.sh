#!/bin/bash

# Function to recursively delete files with specified extensions
delete_files() {
    local folder="$1"
    local file_extension="$2"

    find "$folder" -type f -name "*.$file_extension" -exec rm -f {} +
}

# Specify the folder path
folder_path="docker"

# Delete files with the specified extensions in the folder and subdirectories
delete_files "$folder_path" "log"
delete_files "$folder_path" "mdb"

echo "Deletion complete."

