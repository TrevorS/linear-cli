#!/bin/bash
# ABOUTME: Script to load environment variables from .env file
# ABOUTME: Source this file to set up Linear API key for development

# Check if .env file exists
if [ -f .env ]; then
    export $(cat .env | xargs)
    echo "✓ Linear API key loaded from .env"
else
    echo "✗ No .env file found. Please create one with:"
    echo "  echo 'LINEAR_API_KEY=your_api_key_here' > .env"
fi
