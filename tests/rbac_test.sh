#!/bin/bash
# MCPX Comprehensive RBAC Test Script
# Tests ALL Dashboard API endpoints with all token types

BASE_URL="http://localhost:8080"

# Get M2M tokens
M2M_ADMIN=$(curl -s -X POST $BASE_URL/api/auth/token \
  -d "grant_type=client_credentials&client_id=mcpx_sa_LjZeLB1w3L7ysslG0nIJqOSK0kXIZS1u&client_secret=mcpx_secret_gNRv8IYcdhPaYs4bcoxi1yVFvGLKaGjf1NDoiMyRaNSNQ182" | jq -r '.access_token')

M2M_MEMBER=$(curl -s -X POST $BASE_URL/api/auth/token \
  -d "grant_type=client_credentials&client_id=mcpx_sa_6B4RUdnDeKIdeOQvM4sb9XBW21w9Wvjc&client_secret=mcpx_secret_4LhQr78zuszHnPWbFa1zdlCDP3TUI3hBg3CaKnsFKthvrDx4" | jq -r '.access_token')

PAT_OWNER="mcpx_pat_EzVBLfdJxiPsvSX7DZelTpwnUwc0MpM1"
PAT_MEMBER="mcpx_pat_xinfeb3sZDK6GR0qdkb1X2LbOam58Cm8"

# Function to test endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local description=$3
    local expected_member=$4
    local expected_admin=$5
    local expected_owner=$6
    local data=$7
    
    local url="$BASE_URL$endpoint"
    
    # Test with each token type
    if [ "$method" == "GET" ]; then
        m2m_mem=$(curl -s -o /dev/null -w '%{http_code}' "$url" -H "Authorization: Bearer $M2M_MEMBER")
        m2m_adm=$(curl -s -o /dev/null -w '%{http_code}' "$url" -H "Authorization: Bearer $M2M_ADMIN")
        pat_own=$(curl -s -o /dev/null -w '%{http_code}' "$url" -H "Authorization: Bearer $PAT_OWNER")
        pat_mem=$(curl -s -o /dev/null -w '%{http_code}' "$url" -H "Authorization: Bearer $PAT_MEMBER")
    elif [ "$method" == "POST" ]; then
        m2m_mem=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$url" -H "Authorization: Bearer $M2M_MEMBER" -H "Content-Type: application/json" -d "$data")
        m2m_adm=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$url" -H "Authorization: Bearer $M2M_ADMIN" -H "Content-Type: application/json" -d "$data")
        pat_own=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$url" -H "Authorization: Bearer $PAT_OWNER" -H "Content-Type: application/json" -d "$data")
        pat_mem=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$url" -H "Authorization: Bearer $PAT_MEMBER" -H "Content-Type: application/json" -d "$data")
    elif [ "$method" == "PUT" ]; then
        m2m_mem=$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$url" -H "Authorization: Bearer $M2M_MEMBER" -H "Content-Type: application/json" -d "$data")
        m2m_adm=$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$url" -H "Authorization: Bearer $M2M_ADMIN" -H "Content-Type: application/json" -d "$data")
        pat_own=$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$url" -H "Authorization: Bearer $PAT_OWNER" -H "Content-Type: application/json" -d "$data")
        pat_mem=$(curl -s -o /dev/null -w '%{http_code}' -X PUT "$url" -H "Authorization: Bearer $PAT_MEMBER" -H "Content-Type: application/json" -d "$data")
    elif [ "$method" == "DELETE" ]; then
        m2m_mem=$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$url" -H "Authorization: Bearer $M2M_MEMBER")
        m2m_adm=$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$url" -H "Authorization: Bearer $M2M_ADMIN")
        pat_own=$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$url" -H "Authorization: Bearer $PAT_OWNER")
        pat_mem=$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$url" -H "Authorization: Bearer $PAT_MEMBER")
    fi
    
    # Check results
    check() {
        if [ "$1" == "$2" ]; then echo "✅$1"; else echo "❌$1(exp:$2)"; fi
    }
    
    echo "| $method $endpoint | $(check $m2m_mem $expected_member) | $(check $m2m_adm $expected_admin) | $(check $pat_own $expected_owner) | $(check $pat_mem $expected_member) |"
}

echo "# MCPX Dashboard API RBAC Test Report"
echo "Generated: $(date)"
echo ""
echo "## Token Types"
echo "- M2M Member: Service account with role=member"
echo "- M2M Admin: Service account with role=admin"
echo "- PAT Owner: Personal Access Token for user with role=owner"
echo "- PAT Member: Personal Access Token for user with role=member"
echo ""
echo "## Expected Behavior"
echo "- READ endpoints: All roles return 200"
echo "- WRITE endpoints: Member returns 403, Admin/Owner return success"
echo "- OWNER_ONLY endpoints: Only Owner returns success"
echo ""
echo "## Test Results"
echo ""
echo "| Endpoint | M2M Member | M2M Admin | PAT Owner | PAT Member |"
echo "|----------|------------|-----------|-----------|------------|"

# ============================================
# SERVERS
# ============================================
test_endpoint "GET" "/api/servers" "List servers" "200" "200" "200"
test_endpoint "GET" "/api/servers/weather-server" "Get server" "200" "200" "200"

# ============================================
# GATEWAYS
# ============================================
test_endpoint "GET" "/api/gateways" "List gateways" "200" "200" "200"

# ============================================
# TOKENS (PAT)
# ============================================
test_endpoint "GET" "/api/tokens" "List tokens" "200" "200" "200"

# ============================================
# SERVICE ACCOUNTS
# ============================================
test_endpoint "GET" "/api/service-accounts" "List SAs" "200" "200" "200"

# ============================================
# MEMBERS
# ============================================
test_endpoint "GET" "/api/orgs/members" "List members" "200" "200" "200"

# ============================================
# METRICS
# ============================================
test_endpoint "GET" "/api/metrics/today" "Today metrics" "200" "200" "200"
test_endpoint "GET" "/api/metrics/hourly" "Hourly metrics" "200" "200" "200"
test_endpoint "GET" "/api/metrics/by-target" "By target" "200" "200" "200"
test_endpoint "GET" "/api/metrics/summary" "Summary" "200" "200" "200"

# ============================================
# AUDIT
# ============================================
test_endpoint "GET" "/api/audit" "Audit logs" "200" "200" "200"

# ============================================
# ALERTS
# ============================================
test_endpoint "GET" "/api/alerts" "List alerts" "200" "200" "200"
test_endpoint "GET" "/api/alerts/active" "Active alerts" "200" "200" "200"
test_endpoint "GET" "/api/alerts/history" "Alert history" "200" "200" "200"

# ============================================
# SETTINGS
# ============================================
test_endpoint "GET" "/api/settings" "List settings" "200" "200" "200"
test_endpoint "GET" "/api/limits" "Get limits" "200" "200" "200"

echo ""
echo "## Write Operations (expect 403 for member, 2xx for admin/owner)"
echo ""
echo "| Endpoint | M2M Member | M2M Admin | PAT Owner | PAT Member |"
echo "|----------|------------|-----------|-----------|------------|"

# Create server test
test_endpoint "POST" "/api/servers" "Create server" "403" "201" "201" '{"name":"rbac-test-srv","url":"http://test.local"}'

# Cleanup - delete created servers
curl -s -X DELETE "$BASE_URL/api/servers/rbac-test-srv" -H "Authorization: Bearer $M2M_ADMIN" > /dev/null 2>&1
curl -s -X DELETE "$BASE_URL/api/servers/rbac-test-srv" -H "Authorization: Bearer $PAT_OWNER" > /dev/null 2>&1

echo ""
echo "## MCP Proxy Tests (expect 200 for all authenticated tokens)"
echo ""
echo "| Endpoint | M2M Member | PAT Member |"
echo "|----------|------------|------------|"

# MCP Proxy tests
ORG_SLUG="thiagomendes"
SERVER="weather-server"
MCP_URL="$BASE_URL/mcp/$ORG_SLUG/$SERVER"

m2m_init=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$MCP_URL" \
  -H "Authorization: Bearer $M2M_MEMBER" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}')

pat_init=$(curl -s -o /dev/null -w '%{http_code}' -X POST "$MCP_URL" \
  -H "Authorization: Bearer $PAT_MEMBER" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}')

echo "| POST /mcp/$ORG_SLUG/$SERVER (initialize) | $m2m_init | $pat_init |"

echo ""
echo "---"
echo "Test completed at: $(date)"
