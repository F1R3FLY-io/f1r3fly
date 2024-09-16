#!/bin/bash

#This file is created to test the newly added grammar.
#In the process of generating a parser based on added changes to grammar.js, IntellijIdea does not always pick up the changes because it caches. T
# he only way out for me is to delete the pre-generated files, as well as clear /target.



# Cleaning up the src/main/tree_sitter directory, leaving only grammar.js, GPT-BNFC-to-Tree-Sitter.md, and the tests folder.
DIR="src/main/tree_sitter"

find "$DIR" -mindepth 1 ! -name "grammar.js" ! -name "GPT-BNFC-to-Tree-Sitter.md" ! -name "tests" ! -path "$DIR/tests/*" -exec rm -rf {} +

# Removing the target folder in the root directory if it exists.
TARGET_DIR="target"

if [ -d "$TARGET_DIR" ]; then
    rm -rf "$TARGET_DIR"
    echo "The $TARGET_DIR folder has been successfully removed."
else
    echo "The $TARGET_DIR folder was not found."
fi

echo "Cleanup completed."
