#!/usr/bin/env python3
"""Check commit messages for prohibited words."""

import re
import sys


def main():
    """Check commit message for prohibited patterns."""
    # Get the commit message file path from command line arguments
    if len(sys.argv) < 2:
        print("ERROR: No commit message file provided")
        return 1

    commit_msg_file = sys.argv[1]

    # Read the commit message from the file
    try:
        with open(commit_msg_file, 'r') as f:
            commit_msg = f.read()
    except IOError as e:
        print(f"ERROR: Could not read commit message file: {e}")
        return 1

    # Patterns to check (case-insensitive)
    prohibited_patterns = [
        r'\banthropic\b',
        r'\bclaude\b',
    ]

    for pattern in prohibited_patterns:
        if re.search(pattern, commit_msg, re.IGNORECASE):
            print(f"ERROR: Commit message contains prohibited word matching pattern '{pattern}'")
            print("Please remove references to 'anthropic' or 'claude' from your commit message.")
            return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
