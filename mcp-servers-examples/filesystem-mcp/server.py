#!/usr/bin/env python3
"""
Filesystem MCP Server - FastMCP Implementation with OAuth Client Credentials

A simple MCP server for filesystem operations.
Perfect for testing Virtual Gateways, Session Affinity, and Tool Governance.

AUTHENTICATION: OAuth 2.0 Client Credentials Flow
1. First call POST /token with client_id and client_secret to get access_token
2. Then use Authorization: Bearer <access_token> for MCP calls

Default credentials (configurable via env vars):
  CLIENT_ID: filesystem-client
  CLIENT_SECRET: filesystem-secret-abc123

Tools:
- read_file: Read contents of a file
- write_file: Write contents to a file
- list_directory: List files in a directory
- create_directory: Create a new directory
- delete_file: Delete a file (dangerous - use governance to block in production)
"""

import os
import secrets
import time
from pathlib import Path
from typing import Optional

from fastmcp import FastMCP
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse
from starlette.routing import Route

# OAuth Client Credentials from environment
CLIENT_ID = os.environ.get("CLIENT_ID", "filesystem-client")
CLIENT_SECRET = os.environ.get("CLIENT_SECRET", "filesystem-secret-abc123")

# In-memory token store (for demo purposes)
# In production, use Redis or database
VALID_TOKENS = {}  # token -> expiry_timestamp


def generate_token():
    """Generate a new access token."""
    token = secrets.token_urlsafe(32)
    expires_at = time.time() + 3600  # 1 hour
    VALID_TOKENS[token] = expires_at
    return token, 3600


def validate_token(token: str) -> bool:
    """Check if token is valid and not expired."""
    if token not in VALID_TOKENS:
        return False
    if time.time() > VALID_TOKENS[token]:
        del VALID_TOKENS[token]
        return False
    return True


class OAuthMiddleware(BaseHTTPMiddleware):
    """Middleware to validate OAuth Bearer tokens."""
    
    async def dispatch(self, request, call_next):
        # Skip auth for health checks and token endpoint
        if request.url.path in ["/health", "/token"]:
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
                content={"error": "Authorization header must use Bearer scheme"}
            )
        
        token = auth_header[7:]
        if not validate_token(token):
            return JSONResponse(
                status_code=401,
                content={"error": "Invalid or expired token. Get a new token via POST /token"}
            )
        
        return await call_next(request)


async def token_endpoint(request):
    """OAuth 2.0 Token Endpoint for Client Credentials flow."""
    if request.method != "POST":
        return JSONResponse(
            status_code=405,
            content={"error": "Method not allowed. Use POST."}
        )
    
    try:
        body = await request.json()
    except:
        body = {}
    
    # Also support form data
    if not body:
        form = await request.form()
        body = dict(form)
    
    grant_type = body.get("grant_type", "client_credentials")
    client_id = body.get("client_id")
    client_secret = body.get("client_secret")
    
    if grant_type != "client_credentials":
        return JSONResponse(
            status_code=400,
            content={"error": "unsupported_grant_type", "error_description": "Only client_credentials is supported"}
        )
    
    if client_id != CLIENT_ID or client_secret != CLIENT_SECRET:
        return JSONResponse(
            status_code=401,
            content={"error": "invalid_client", "error_description": "Invalid client_id or client_secret"}
        )
    
    token, expires_in = generate_token()
    
    return JSONResponse({
        "access_token": token,
        "token_type": "Bearer",
        "expires_in": expires_in
    })


# Create MCP server
mcp = FastMCP("filesystem-server", version="1.0.0")

# Base directory for file operations (pod-local)
BASE_DIR = Path("/tmp/mcp-filesystem")
BASE_DIR.mkdir(exist_ok=True)


def sanitize_path(path: str) -> Path:
    """
    Sanitize and validate file path to prevent directory traversal.
    All paths are relative to BASE_DIR.
    """
    clean_path = path.lstrip("/")
    full_path = (BASE_DIR / clean_path).resolve()

    if not str(full_path).startswith(str(BASE_DIR)):
        raise ValueError(f"Path '{path}' is outside allowed directory")

    return full_path


@mcp.tool()
def read_file(path: str) -> str:
    """Read contents of a file.

    Args:
        path: Path to the file to read (relative to base directory)

    Returns:
        File contents and metadata as formatted string
    """
    try:
        file_path = sanitize_path(path)

        if not file_path.exists():
            return f"Error: File not found: {path}"

        if not file_path.is_file():
            return f"Error: Path is not a file: {path}"

        content = file_path.read_text()
        size = len(content)

        return f"""File: {path}
Size: {size} bytes
Content:
{content}"""
    except Exception as e:
        return f"Error reading file: {str(e)}"


@mcp.tool()
def write_file(path: str, content: str) -> str:
    """Write contents to a file.

    Args:
        path: Path to the file to write (relative to base directory)
        content: Content to write to the file

    Returns:
        Success message with file info
    """
    try:
        file_path = sanitize_path(path)
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.write_text(content)
        size = len(content)

        return f"""File written successfully
Path: {path}
Size: {size} bytes
Message: File written successfully"""
    except Exception as e:
        return f"Error writing file: {str(e)}"


@mcp.tool()
def list_directory(path: str = ".") -> str:
    """List files and directories in a path.

    Args:
        path: Path to the directory to list (default: current directory)

    Returns:
        Formatted list of files and directories
    """
    try:
        dir_path = sanitize_path(path)

        if not dir_path.exists():
            return f"Error: Directory not found: {path}"

        if not dir_path.is_dir():
            return f"Error: Path is not a directory: {path}"

        entries = []
        for item in sorted(dir_path.iterdir()):
            item_type = "directory" if item.is_dir() else "file"
            size = item.stat().st_size if item.is_file() else 0
            entries.append(f"  - {item.name} ({item_type}, {size} bytes)")

        result = f"Directory: {path}\n"
        result += f"Total entries: {len(entries)}\n\n"
        result += "Entries:\n"
        result += "\n".join(entries) if entries else "  (empty)"

        return result
    except Exception as e:
        return f"Error listing directory: {str(e)}"


@mcp.tool()
def create_directory(path: str) -> str:
    """Create a new directory.

    Args:
        path: Path to the directory to create

    Returns:
        Success message
    """
    try:
        dir_path = sanitize_path(path)

        if dir_path.exists():
            return f"Error: Path already exists: {path}"

        dir_path.mkdir(parents=True)

        return f"""Directory created successfully
Path: {path}
Message: Directory created successfully"""
    except Exception as e:
        return f"Error creating directory: {str(e)}"


@mcp.tool()
def delete_file(path: str) -> str:
    """Delete a file (DANGEROUS: should be blocked by governance in production).

    Args:
        path: Path to the file to delete

    Returns:
        Success message
    """
    try:
        file_path = sanitize_path(path)

        if not file_path.exists():
            return f"Error: File not found: {path}"

        if not file_path.is_file():
            return f"Error: Path is not a file: {path}"

        file_path.unlink()

        return f"""File deleted successfully
Path: {path}
Message: File deleted successfully"""
    except Exception as e:
        return f"Error deleting file: {str(e)}"


if __name__ == "__main__":
    import uvicorn
    from starlette.applications import Starlette
    from starlette.routing import Route, Mount
    
    print("=" * 70)
    print("Filesystem MCP Server - FastMCP (Python) with OAuth Client Credentials")
    print("=" * 70)
    print("AUTHENTICATION: OAuth 2.0 Client Credentials Flow")
    print("")
    print("Step 1 - Get token:")
    print(f"  POST /token")
    print(f"  Body: {{'client_id': '{CLIENT_ID}', 'client_secret': '{CLIENT_SECRET}', 'grant_type': 'client_credentials'}}")
    print("")
    print("Step 2 - Use token:")
    print("  Authorization: Bearer <access_token>")
    print("")
    print("Tools (5 total):")
    print("")
    print("  Safe Tools (read-only):")
    print("    1. read_file        - Read file contents")
    print("    3. list_directory   - List directory entries")
    print("")
    print("  Moderate Risk Tools:")
    print("    2. write_file       - Write to file (creates files)")
    print("    4. create_directory - Create directory")
    print("")
    print("  Dangerous Tools (governance should block in production):")
    print("    5. delete_file      - Delete file")
    print("")
    print(f"Base directory: {BASE_DIR}")
    print("")
    print("Starting server on port 8000...")
    print("=" * 70)

    # Get the FastMCP Starlette app and add middleware + token endpoint
    mcp_app = mcp.get_app(transport="streamable-http", path="/mcp")
    
    # Create a new app with token endpoint
    app = Starlette(
        routes=[
            Route("/token", token_endpoint, methods=["POST"]),
            Mount("/", app=mcp_app),
        ]
    )
    app.add_middleware(OAuthMiddleware)
    
    uvicorn.run(app, host="0.0.0.0", port=8000)
