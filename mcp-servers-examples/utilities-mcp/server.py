#!/usr/bin/env python3
"""
Utilities MCP Server - Full Stateless with Bearer Token Auth

A Model Context Protocol server providing pure utility functions.

AUTHENTICATION: Authorization: Bearer <token> header required
Default token: utilities-token-secret-67890 (configurable via BEARER_TOKEN env var)

CRITICAL: This server is configured as FULL STATELESS (stateless_http=True)
- Does NOT return mcp-session-id on initialize
- Does NOT require mcp-session-id on subsequent calls
- All operations are pure functions without state
"""

import os
import base64
import hashlib
import uuid
import json
import re

from fastmcp import FastMCP
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse

# Bearer Token from environment
BEARER_TOKEN = os.environ.get("BEARER_TOKEN", "utilities-token-secret-67890")


class BearerTokenMiddleware(BaseHTTPMiddleware):
    """Middleware to validate Authorization: Bearer <token> header."""
    
    async def dispatch(self, request, call_next):
        # Skip auth for health checks
        if request.url.path == "/health":
            return await call_next(request)
        
        auth_header = request.headers.get("Authorization")
        if not auth_header:
            return JSONResponse(
                status_code=401,
                content={"error": "Missing Authorization header"}
            )
        
        if not auth_header.startswith("Bearer "):
            return JSONResponse(
                status_code=401,
                content={"error": "Authorization header must start with 'Bearer '"}
            )
        
        token = auth_header[7:]  # Remove "Bearer " prefix
        if token != BEARER_TOKEN:
            return JSONResponse(
                status_code=403,
                content={"error": "Invalid bearer token"}
            )
        
        return await call_next(request)


# Create FastMCP server
mcp = FastMCP("Utilities MCP Server")


@mcp.tool()
def encode_base64(text: str) -> str:
    """
    Encode string to base64.

    Args:
        text: String to encode

    Returns:
        Base64 encoded string

    Example:
        encode_base64("Hello World") -> "SGVsbG8gV29ybGQ="
    """
    try:
        encoded = base64.b64encode(text.encode('utf-8')).decode('utf-8')
        return encoded
    except Exception as e:
        raise ValueError(f"Failed to encode: {str(e)}")


@mcp.tool()
def decode_base64(encoded: str) -> str:
    """
    Decode base64 to string.

    Args:
        encoded: Base64 encoded string

    Returns:
        Decoded string

    Example:
        decode_base64("SGVsbG8gV29ybGQ=") -> "Hello World"
    """
    try:
        decoded = base64.b64decode(encoded.encode('utf-8')).decode('utf-8')
        return decoded
    except Exception as e:
        raise ValueError(f"Failed to decode: {str(e)}")


@mcp.tool()
def generate_uuid() -> str:
    """
    Generate random UUID v4.

    Returns:
        UUID string in format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx

    Example:
        generate_uuid() -> "550e8400-e29b-41d4-a716-446655440000"
    """
    return str(uuid.uuid4())


@mcp.tool()
def hash_text(text: str, algorithm: str = "sha256") -> str:
    """
    Generate hash using specified algorithm.

    Args:
        text: Text to hash
        algorithm: Hash algorithm (md5, sha256, sha512)

    Returns:
        Hexadecimal hash string

    Example:
        hash_text("password", "sha256") -> "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"
    """
    algorithm = algorithm.lower()

    try:
        if algorithm == "md5":
            return hashlib.md5(text.encode('utf-8')).hexdigest()
        elif algorithm == "sha256":
            return hashlib.sha256(text.encode('utf-8')).hexdigest()
        elif algorithm == "sha512":
            return hashlib.sha512(text.encode('utf-8')).hexdigest()
        else:
            raise ValueError(f"Unsupported algorithm: {algorithm}. Use: md5, sha256, sha512")
    except Exception as e:
        raise ValueError(f"Failed to hash: {str(e)}")


@mcp.tool()
def reverse_string(text: str) -> str:
    """
    Reverse a string.

    Args:
        text: String to reverse

    Returns:
        Reversed string

    Example:
        reverse_string("Hello") -> "olleH"
    """
    return text[::-1]


@mcp.tool()
def validate_email(email: str) -> dict:
    """
    Validate email format using regex.

    Args:
        email: Email address to validate

    Returns:
        Dictionary with 'valid' (bool) and 'reason' (str)

    Example:
        validate_email("user@example.com") -> {"valid": true, "reason": "Valid email format"}
        validate_email("invalid") -> {"valid": false, "reason": "Invalid email format"}
    """
    # RFC 5322 simplified pattern
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    is_valid = bool(re.match(pattern, email))

    return {
        "valid": is_valid,
        "reason": "Valid email format" if is_valid else "Invalid email format"
    }


@mcp.tool()
def format_json(json_str: str, indent: int = 2) -> str:
    """
    Pretty-print JSON with indentation.

    Args:
        json_str: JSON string to format
        indent: Number of spaces for indentation (default: 2)

    Returns:
        Formatted JSON string

    Example:
        format_json('{"name":"John","age":30}') ->
        {
          "name": "John",
          "age": 30
        }
    """
    try:
        obj = json.loads(json_str)
        return json.dumps(obj, indent=indent, ensure_ascii=False)
    except json.JSONDecodeError as e:
        raise ValueError(f"Invalid JSON: {str(e)}")


if __name__ == "__main__":
    import uvicorn
    
    print("=" * 70)
    print("Utilities MCP Server - Full Stateless with Bearer Token Auth")
    print("=" * 70)
    print(f"AUTHENTICATION: Authorization: Bearer <token> required")
    print(f"Token: {BEARER_TOKEN}")
    print("")
    print("Tools (7 pure utility functions):")
    print("  1. encode_base64    - Encode string to base64")
    print("  2. decode_base64    - Decode base64 to string")
    print("  3. generate_uuid    - Generate random UUID v4")
    print("  4. hash_text        - Hash using MD5/SHA256/SHA512")
    print("  5. reverse_string   - Reverse a string")
    print("  6. validate_email   - Validate email format")
    print("  7. format_json      - Pretty-print JSON")
    print("")
    print("STATELESS Configuration:")
    print("  - stateless_http=True")
    print("  - NO mcp-session-id in responses")
    print("  - NO session required for any operation")
    print("  - All functions are pure and idempotent")
    print("")
    print("Starting server on port 8000...")
    print("=" * 70)

    # Get the FastMCP Starlette app and add middleware
    app = mcp.get_app(transport="streamable-http", path="/mcp", stateless_http=True)
    app.add_middleware(BearerTokenMiddleware)
    
    uvicorn.run(app, host="0.0.0.0", port=8000)
