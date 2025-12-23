#!/bin/bash
# MCPX Load Test Script with configurable profiles
# Usage: ./load-test.sh [low|medium|high]

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
USER_ID="${USER_ID:-e4f1c4f9-e421-40a1-bf41-cc53efc3b7f6}"
SERVER_NAME="${SERVER_NAME:-example-server}"
PROXY_URL="$BASE_URL/mcp/$USER_ID/$SERVER_NAME"

# Load profiles
PROFILE="${1:-low}"
case "$PROFILE" in
    low)    REQUESTS_PER_TOOL=100 ;;
    medium) REQUESTS_PER_TOOL=500 ;;
    high)   REQUESTS_PER_TOOL=1000 ;;
    *)      echo "Usage: $0 [low|medium|high]"; exit 1 ;;
esac

# Tools to call
TOOLS="echo add sampleLLM getTinyImage longRunningOperation"
PARALLEL_JOBS=5  # Number of parallel curl calls per tool

echo "🚀 MCPX Load Test"
echo "   Profile: $PROFILE ($REQUESTS_PER_TOOL requests/tool)"
echo "   URL: $PROXY_URL"
echo "   Tools: $TOOLS"
echo ""

# Step 1: Initialize
echo "1. Initialize..."
SESSION_ID=$(curl -s -i --max-time 15 -X POST "$PROXY_URL" \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"load-test","version":"1.0"}}}' \
    | grep -i "mcp-session-id:" | sed 's/mcp-session-id: //i' | tr -d '\r\n')

if [ -z "$SESSION_ID" ]; then
    echo "   ✗ Failed to get session"
    exit 1
fi
echo "   ✓ Session: $SESSION_ID"

# Initialized notification
curl -s --max-time 5 -X POST "$PROXY_URL" \
    -H "Content-Type: application/json" \
    -H "mcp-session-id: $SESSION_ID" \
    -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null

# Step 2: Function to call a tool N times
call_tool() {
    local tool=$1
    local count=$2
    local session=$3
    local url=$4
    local success=0
    local failed=0
    
    for ((i=1; i<=count; i++)); do
        http_code=$(curl -s -o /dev/null -w "%{http_code}" --max-time 30 -X POST "$url" \
            -H "Content-Type: application/json" \
            -H "mcp-session-id: $session" \
            -d "{\"jsonrpc\":\"2.0\",\"id\":$i,\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":{}}}" 2>/dev/null || echo "000")
        
        if [[ "$http_code" == "200" ]]; then
            ((success++))
        else
            ((failed++))
        fi
        
        # Progress indicator every 10 requests
        if ((i % 10 == 0)); then
            echo "     $tool: $i/$count (✓$success ✗$failed)"
        fi
    done
    
    echo "   [$tool] Done: ✓$success ✗$failed"
}

export -f call_tool

# Step 3: Run tools in parallel
echo ""
echo "2. Calling tools in parallel..."
echo "   Each tool: $REQUESTS_PER_TOOL requests"
echo ""

start_time=$(date +%s)

# Run each tool in background (parallel between tools)
pids=()
for tool in $TOOLS; do
    call_tool "$tool" "$REQUESTS_PER_TOOL" "$SESSION_ID" "$PROXY_URL" &
    pids+=($!)
done

# Wait for all to complete
for pid in ${pids[@]}; do
    wait $pid
done

end_time=$(date +%s)
duration=$((end_time - start_time))

# Calculate totals
total_tools=$(echo $TOOLS | wc -w)
total_requests=$((REQUESTS_PER_TOOL * total_tools))

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Load Test Complete!"
echo ""
echo "   Profile:    $PROFILE"
echo "   Tools:      $total_tools"
echo "   Requests:   $total_requests total"
echo "   Duration:   ${duration}s"
echo "   Rate:       ~$((total_requests / (duration + 1))) req/s"
echo ""
echo "📊 Check dashboard: $BASE_URL/dashboard"
echo ""
