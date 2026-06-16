#/usr/bin/bash

CREATED_TIME=$(git log --reverse --format="%at" | head -n 1)
CURRENT_TIME=$(date +%s)
DIFF_DAYS=$(( (CURRENT_TIME - CREATED_TIME) / 86400 ))

echo "The repository has been active for: $DIFF_DAYS days"