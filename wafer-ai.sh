#!/bin/bash
# Wafer AI API helper script
# Usage: ./wafer-ai.sh "Your prompt here"

API_KEY="${WAFER_API_KEY:-}"
MODEL="${WAFER_MODEL:-GLM-5.1}"
MAX_TOKENS="${WAFER_MAX_TOKENS:-128}"

if [ -z "$API_KEY" ]; then
    echo "Error: WAFER_API_KEY environment variable is not set."
    echo "Set it with: export WAFER_API_KEY='your-api-key'"
    exit 1
fi

if [ -z "$1" ]; then
    echo "Usage: $0 \"Your prompt here\""
    exit 1
fi

PROMPT="$1"

curl -sS "https://pass.wafer.ai/v1/chat/completions" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"$MODEL\",
    \"messages\": [{\"role\": \"user\", \"content\": \"$PROMPT\"}],
    \"max_tokens\": $MAX_TOKENS
  }"
