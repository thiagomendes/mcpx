#!/usr/bin/env python3
import requests
import json
import sys

BASE_URL = "http://localhost:8080/mcp/tm-dev-lab-org/tmdevlab-gateway"

def run_test():
    # Headers based on GOVERNANCE_TEST_SCRIPT.md
    common_headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream"
    }
    
    # 1. Initialize to get Session ID
    print("1. Sending 'initialize'...")
    init_payload = {
        "jsonrpc": "2.0",
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        },
        "id": 1
    }
    
    try:
        resp = requests.post(BASE_URL, json=init_payload, headers=common_headers)
        print(f"   Response Status: {resp.status_code}")
        
        session_id = resp.headers.get("mcp-session-id")
        if not session_id:
            print("   ERROR: No 'mcp-session-id' header found in response.")
            print("   Headers:", dict(resp.headers))
            return False
            
        print(f"   Session ID acquired: {session_id}")
        
        # 2. Call Tool with Session ID
        print("\n2. Sending 'tools/call' with Session ID...")
        tool_payload = {
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "tmdevlab_get_weather",
                "arguments": {"city": "Paris"}
            },
            "id": 2
        }
        
        headers_with_session = {
            **common_headers,
            "Mcp-Session-Id": session_id,
        }
        
        tool_resp = requests.post(BASE_URL, json=tool_payload, headers=headers_with_session)
        
        print(f"   Response Status: {tool_resp.status_code}")
        print(f"   Response Body: {tool_resp.text[:500]}...")
        
        if tool_resp.status_code == 200:
            print("\nSUCCESS: Tool call executed successfully.")
            return True
        else:
            print(f"\nFAILURE: Tool call failed with status {tool_resp.status_code}")
            return False
            
    except Exception as e:
        print(f"\nEXCEPTION: {e}")
        return False

if __name__ == "__main__":
    success = run_test()
    if not success:
        sys.exit(1)
