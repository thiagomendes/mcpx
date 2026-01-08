#!/bin/bash
# Test script to trigger alert rules
# Simulates: high latency (deepwiki) and error rate (bad inputs)

PAT="${PAT:-mcpx_pat_CHANGEME}"
BASE_URL="${BASE_URL:-http://localhost:8080}"
ORG_SLUG="${ORG_SLUG:-mailthiagomendes-9855fe5e}"

DEEPWIKI_URL="$BASE_URL/mcp/$ORG_SLUG/deepwiki"
EXAMPLE_URL="$BASE_URL/mcp/$ORG_SLUG/example-server"

echo "🚨 Alert Test Script"
echo "===================="
echo "   PAT: ${PAT:0:20}..."
echo "   BASE_URL: $BASE_URL"
echo "   ORG_SLUG: $ORG_SLUG"
echo ""

# Function to initialize MCP session
init_session() {
    local url=$1
    local session=$(curl -s -i --max-time 30 -X POST "$url" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $PAT" \
        -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"alert-test","version":"1.0"}}}' \
        | grep -i "mcp-session-id:" | sed 's/mcp-session-id: //i' | tr -d '\r\n')
    echo "$session"
}

# Function to call tool
call_tool() {
    local url=$1
    local session=$2
    local tool=$3
    local args=$4
    
    curl -s --max-time 60 -X POST "$url" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $PAT" \
        -H "mcp-session-id: $session" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":$args}}"
}

# ============================================
# TEST 1: High Latency (DeepWiki is slow)
# ============================================
echo "📊 TEST 1: High Latency Simulation (DeepWiki)"
echo "   This server is naturally slow, triggering latency alerts..."
echo ""

echo "   Initializing DeepWiki session..."
DEEPWIKI_SESSION=$(init_session "$DEEPWIKI_URL")
if [ -z "$DEEPWIKI_SESSION" ]; then
    echo "   ✗ Failed to get DeepWiki session"
else
    echo "   ✓ Session: ${DEEPWIKI_SESSION:0:20}..."
    
    # Send initialized notification
    curl -s --max-time 5 -X POST "$DEEPWIKI_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $PAT" \
        -H "mcp-session-id: $DEEPWIKI_SESSION" \
        -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null
    
    echo "   Calling DeepWiki tools (5 parallel calls to trigger latency alert)..."
    
    # Make 5 parallel calls for faster testing
    for i in {1..5}; do
        (
            result=$(call_tool "$DEEPWIKI_URL" "$DEEPWIKI_SESSION" "read_wiki_structure" '{"repoUrl":"https://github.com/anthropics/anthropic-cookbook"}')
            echo "     Call $i done"
        ) &
    done
    wait
    echo "   ✓ All DeepWiki calls completed"
fi

echo ""

# ============================================
# TEST 2: Error Rate (Bad Inputs)
# ============================================
echo "📊 TEST 2: Error Rate Simulation (Bad Tool Calls)"
echo "   Sending invalid inputs to trigger error rate alerts..."
echo ""

echo "   Initializing example-server session..."
EXAMPLE_SESSION=$(init_session "$EXAMPLE_URL")
if [ -z "$EXAMPLE_SESSION" ]; then
    echo "   ✗ Failed to get example-server session"
else
    echo "   ✓ Session: ${EXAMPLE_SESSION:0:20}..."
    
    # Send initialized notification
    curl -s --max-time 5 -X POST "$EXAMPLE_URL" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $PAT" \
        -H "mcp-session-id: $EXAMPLE_SESSION" \
        -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' > /dev/null
    
    echo "   Sending bad tool calls..."
    
    # Call non-existent tools
    for i in {1..15}; do
        call_tool "$EXAMPLE_URL" "$EXAMPLE_SESSION" "nonexistent_tool_$i" '{"invalid":"params"}' > /dev/null
    done
    echo "     ✓ 15 bad tool calls sent"
    
    # Call valid tools with wrong params
    for i in {1..15}; do
        call_tool "$EXAMPLE_URL" "$EXAMPLE_SESSION" "echo" '{"wrong_param":"value"}' > /dev/null
    done
    echo "     ✓ 15 wrong params calls sent"
fi

echo ""
echo "============================================"
echo "✅ Test Complete!"
echo ""
echo "Wait ~60 seconds for alert_eval job to run"
echo "Then check worker logs:"
echo "  docker logs mcpx-worker --tail 30"
echo ""
echo "Check alerts in dashboard:"
echo "  http://localhost:3000/alerts"
echo ""
