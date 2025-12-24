#!/usr/bin/env python3
"""
Weather MCP Server - FastMCP Implementation with API Key Auth
Demonstrates MCP Streamable HTTP with SSE streaming for long-running operations.

AUTHENTICATION: X-API-Key header required
Default key: weather-api-key-12345 (configurable via API_KEY env var)

Implements 6 tools:
- 5 stateless tools (return JSON immediately)
- 1 streaming tool (returns SSE stream with progressive updates)
"""

import asyncio
import os
import random
from datetime import datetime, timedelta
from typing import Optional

from fastmcp import FastMCP, Context
from starlette.middleware.base import BaseHTTPMiddleware
from starlette.responses import JSONResponse

# API Key from environment
API_KEY = os.environ.get("API_KEY", "weather-api-key-12345")


class APIKeyMiddleware(BaseHTTPMiddleware):
    """Middleware to validate X-API-Key header."""
    
    async def dispatch(self, request, call_next):
        # Skip auth for health checks
        if request.url.path == "/health":
            return await call_next(request)
        
        api_key = request.headers.get("X-API-Key")
        if not api_key:
            return JSONResponse(
                status_code=401,
                content={"error": "Missing X-API-Key header"}
            )
        
        if api_key != API_KEY:
            return JSONResponse(
                status_code=403,
                content={"error": "Invalid API key"}
            )
        
        return await call_next(request)


# Create MCP server
mcp = FastMCP("weather-server", version="3.0.0")

# Mock weather database
WEATHER_DATABASE = {
    "new york": {
        "city": "New York",
        "country": "USA",
        "lat": 40.7128,
        "lon": -74.0060,
        "temperature": 22.5,
        "feels_like": 24.0,
        "conditions": "Partly Cloudy",
        "humidity": 65,
        "pressure": 1013,
        "wind_speed": 15.0,
        "wind_dir": "NW",
        "uv_index": 6,
        "visibility": 10,
    },
    "london": {
        "city": "London",
        "country": "UK",
        "lat": 51.5074,
        "lon": -0.1278,
        "temperature": 15.0,
        "feels_like": 13.0,
        "conditions": "Rainy",
        "humidity": 85,
        "pressure": 1008,
        "wind_speed": 20.0,
        "wind_dir": "SW",
        "uv_index": 2,
        "visibility": 5,
    },
    "tokyo": {
        "city": "Tokyo",
        "country": "Japan",
        "lat": 35.6762,
        "lon": 139.6503,
        "temperature": 28.0,
        "feels_like": 30.0,
        "conditions": "Sunny",
        "humidity": 55,
        "pressure": 1015,
        "wind_speed": 10.0,
        "wind_dir": "E",
        "uv_index": 8,
        "visibility": 10,
    },
    "sydney": {
        "city": "Sydney",
        "country": "Australia",
        "lat": -33.8688,
        "lon": 151.2093,
        "temperature": 20.0,
        "feels_like": 19.0,
        "conditions": "Clear",
        "humidity": 60,
        "pressure": 1018,
        "wind_speed": 12.0,
        "wind_dir": "SE",
        "uv_index": 7,
        "visibility": 10,
    },
    "paris": {
        "city": "Paris",
        "country": "France",
        "lat": 48.8566,
        "lon": 2.3522,
        "temperature": 18.0,
        "feels_like": 17.0,
        "conditions": "Cloudy",
        "humidity": 70,
        "pressure": 1010,
        "wind_speed": 8.0,
        "wind_dir": "W",
        "uv_index": 4,
        "visibility": 8,
    },
    "san francisco": {
        "city": "San Francisco",
        "country": "USA",
        "lat": 37.7749,
        "lon": -122.4194,
        "temperature": 16.0,
        "feels_like": 15.0,
        "conditions": "Foggy",
        "humidity": 75,
        "pressure": 1012,
        "wind_speed": 18.0,
        "wind_dir": "W",
        "uv_index": 5,
        "visibility": 6,
    },
}


def get_city_weather(city: str) -> dict:
    """Get weather data for a city, with fallback for unknown cities."""
    city_lower = city.lower().strip()
    if city_lower in WEATHER_DATABASE:
        return WEATHER_DATABASE[city_lower]
    else:
        return {
            "city": city.title(),
            "country": "Unknown",
            "lat": 0.0,
            "lon": 0.0,
            "temperature": 20.0,
            "feels_like": 20.0,
            "conditions": "Unknown",
            "humidity": 50,
            "pressure": 1013,
            "wind_speed": 0.0,
            "wind_dir": "N",
            "uv_index": 5,
            "visibility": 10,
        }


# =============================================================================
# Tool 1: get-weather (Stateless - returns JSON immediately)
# =============================================================================
@mcp.tool()
def get_weather(city: str) -> str:
    """Get current weather information for a city including temperature, conditions, humidity, wind, and more.

    Args:
        city: Name of the city (e.g., 'New York', 'London', 'Tokyo')
    """
    weather = get_city_weather(city)
    now = datetime.utcnow().isoformat() + "Z"

    return f"""Weather in {weather['city']}, {weather['country']}
Temperature: {weather['temperature']}°C (feels like {weather['feels_like']}°C)
Conditions: {weather['conditions']}
Humidity: {weather['humidity']}%
Pressure: {weather['pressure']} hPa
Wind: {weather['wind_speed']} km/h {weather['wind_dir']}
UV Index: {weather['uv_index']}
Visibility: {weather['visibility']} km
Coordinates: {weather['lat']}°N, {weather['lon']}°E
Updated: {now}"""


# =============================================================================
# Tool 2: get-forecast (Stateless - returns JSON immediately)
# =============================================================================
@mcp.tool()
def get_forecast(city: str, days: int = 5) -> str:
    """Get weather forecast for the next few days (up to 5 days).

    Args:
        city: Name of the city
        days: Number of days to forecast (1-5, default: 5)
    """
    if days < 1:
        days = 1
    if days > 5:
        days = 5

    weather = get_city_weather(city)
    result = f"Weather forecast for {weather['city']}, {weather['country']} ({days} days):\n\n"

    for i in range(1, days + 1):
        date = (datetime.now() + timedelta(days=i)).strftime("%a, %b %d")
        temp = weather["temperature"] + random.randint(-2, 2)
        condition = random.choice(["Sunny", "Partly Cloudy", "Cloudy", "Rainy", "Clear"])
        precipitation = random.randint(0, 60)

        result += f"Day {i} ({date}):\n"
        result += f"  High: {temp + 3}°C, Low: {temp - 3}°C\n"
        result += f"  Conditions: {condition}\n"
        result += f"  Precipitation: {precipitation}%\n\n"

    return result


# =============================================================================
# Tool 3: search-location (Stateless - returns JSON immediately)
# =============================================================================
@mcp.tool()
def search_location(query: str) -> str:
    """Search for a location and get its coordinates.

    Args:
        query: Search query (city name or partial match)
    """
    query_lower = query.lower().strip()
    matches = []

    for key, weather in WEATHER_DATABASE.items():
        if query_lower in key or query_lower in weather["country"].lower():
            matches.append(
                f"{weather['city']}, {weather['country']} ({weather['lat']}°N, {weather['lon']}°E)"
            )

    if not matches:
        return f"No locations found matching '{query}'"

    result = f"Found {len(matches)} location(s) matching '{query}':\n\n"
    for match in matches:
        result += f"- {match}\n"

    return result


# =============================================================================
# Tool 4: get-air-quality (Stateless - returns JSON immediately)
# =============================================================================
@mcp.tool()
def get_air_quality(city: str) -> str:
    """Get air quality information including AQI and pollutant levels.

    Args:
        city: Name of the city
    """
    weather = get_city_weather(city)
    now = datetime.utcnow().isoformat() + "Z"

    # Mock AQI data
    aqi = 50 + random.randint(0, 100)
    if aqi <= 50:
        quality = "Good"
    elif aqi <= 100:
        quality = "Moderate"
    elif aqi <= 150:
        quality = "Unhealthy for Sensitive Groups"
    else:
        quality = "Unhealthy"

    return f"""Air Quality in {weather['city']}, {weather['country']}
AQI: {aqi} ({quality})
PM2.5: {aqi * 0.5:.1f} µg/m³
PM10: {aqi * 0.8:.1f} µg/m³
O3: {aqi * 0.3:.1f} ppb
NO2: {aqi * 0.2:.1f} ppb
CO: {aqi * 0.1:.1f} ppm
Updated: {now}"""


# =============================================================================
# Tool 5: get-weather-alerts (Stateless - returns JSON immediately)
# =============================================================================
@mcp.tool()
def get_weather_alerts(city: str) -> str:
    """Get active weather alerts and warnings for a city.

    Args:
        city: Name of the city
    """
    weather = get_city_weather(city)
    now = datetime.utcnow().isoformat() + "Z"
    expires = (datetime.utcnow() + timedelta(hours=24)).isoformat() + "Z"

    # Mock alerts (random)
    has_alert = random.randint(0, 2) == 0

    if not has_alert:
        return f"No active weather alerts for {weather['city']}, {weather['country']}"

    alert_types = [
        "Heavy Rain Warning",
        "High Wind Advisory",
        "Heat Advisory",
        "Winter Weather Advisory",
        "Flood Watch",
    ]
    alert = random.choice(alert_types)

    return f"""Active Weather Alert for {weather['city']}, {weather['country']}

Alert Type: {alert}
Severity: Moderate
Effective: {now}
Expires: {expires}

Description: Monitor conditions and stay informed through local weather updates."""


# =============================================================================
# Tool 6: monitor_weather - Long-running weather monitoring with SSE streaming
# =============================================================================
@mcp.tool()
async def monitor_weather(city: str, duration: int = 10, ctx: Context = None) -> str:
    """Monitor weather conditions over time with progressive updates via SSE stream.

    This tool demonstrates Streamable HTTP with SSE streaming:
    - Long-running operation (10-60 seconds)
    - Progressive updates sent via text/event-stream response
    - Server pushes notifications during execution using Context
    - Client receives real-time updates

    Args:
        city: Name of the city to monitor
        duration: Duration in seconds to monitor (default: 10, max: 60)

    Returns:
        Summary of weather monitoring session with all updates
    """
    # Validate duration
    if duration < 1:
        duration = 1
    if duration > 60:
        duration = 60

    weather = get_city_weather(city)
    start_time = datetime.utcnow()

    # Determine update interval
    update_interval = 3 if duration > 10 else 2
    updates = int(duration / update_interval)
    if updates < 1:
        updates = 1

    all_updates = []

    # Send progressive updates via SSE stream
    for i in range(updates):
        # Simulate temperature fluctuation
        temp_delta = random.uniform(-0.5, 0.5)
        current_temp = weather["temperature"] + temp_delta

        # Simulate varying conditions
        conditions = weather["conditions"]
        if random.randint(0, 10) > 7:
            conditions = random.choice(["Sunny", "Cloudy", "Partly Cloudy", "Rainy"])

        elapsed = (datetime.utcnow() - start_time).total_seconds()
        update_msg = f"[+{int(elapsed):02d}s] Update {i+1}/{updates}: Temp {current_temp:.1f}°C, {conditions}"
        all_updates.append(update_msg)

        # Send progress notification via SSE stream using Context
        if ctx:
            await ctx.report_progress(
                progress=i + 1,
                total=updates,
            )
            # Send info notification
            await ctx.info(update_msg)

        # Sleep between updates
        if i < updates - 1:
            await asyncio.sleep(update_interval)

    end_time = datetime.utcnow()
    total_duration = (end_time - start_time).total_seconds()

    # Build final result
    result = f"Weather Monitoring Complete\n"
    result += f"Location: {weather['city']}, {weather['country']}\n"
    result += f"Duration: {total_duration:.1f} seconds\n"
    result += f"Started: {start_time.isoformat()}Z\n"
    result += f"Completed: {end_time.isoformat()}Z\n\n"
    result += "Updates Received:\n"
    result += "\n".join(all_updates)
    result += f"\n\nTotal updates: {len(all_updates)}"

    return result


# =============================================================================
# Tool 7: track_storm - Simulate storm tracking with extended SSE monitoring
# =============================================================================
@mcp.tool()
async def track_storm(
    location: str, duration: int = 20, update_frequency: int = 5, ctx: Context = None
) -> str:
    """Track storm development with real-time SSE updates.

    Simulates tracking a storm system with progressive position and intensity updates.
    Demonstrates extended streaming operations with FastMCP Streamable HTTP.

    Args:
        location: Location to track storm from
        duration: Duration in seconds to track (default: 20, max: 120)
        update_frequency: Number of updates to send (default: 5, max: 20)

    Returns:
        Storm tracking summary with all recorded updates
    """
    # Validate parameters
    if duration < 5:
        duration = 5
    if duration > 120:
        duration = 120
    if update_frequency < 2:
        update_frequency = 2
    if update_frequency > 20:
        update_frequency = 20

    start_time = datetime.utcnow()
    update_interval = duration / update_frequency

    # Initial storm data
    storm_lat = random.uniform(25.0, 45.0)
    storm_lon = random.uniform(-120.0, -70.0)
    intensity = random.randint(40, 65)  # mph

    all_updates = []

    # Simulate storm movement and intensity changes
    for i in range(update_frequency):
        # Update position (moving northeast)
        storm_lat += random.uniform(0.1, 0.3)
        storm_lon += random.uniform(0.1, 0.3)

        # Update intensity
        intensity += random.randint(-5, 10)
        intensity = max(35, min(120, intensity))  # Keep in realistic range

        # Classify storm
        if intensity < 39:
            category = "Tropical Depression"
        elif intensity < 74:
            category = "Tropical Storm"
        elif intensity < 96:
            category = "Category 1 Hurricane"
        elif intensity < 111:
            category = "Category 2 Hurricane"
        else:
            category = "Category 3 Hurricane"

        elapsed = (datetime.utcnow() - start_time).total_seconds()
        update_msg = (
            f"[+{int(elapsed):03d}s] Update {i+1}/{update_frequency}: "
            f"{category} at ({storm_lat:.2f}°, {storm_lon:.2f}°), "
            f"Wind {intensity} mph"
        )
        all_updates.append(update_msg)

        # Send progress and info via SSE stream
        if ctx:
            await ctx.report_progress(progress=i + 1, total=update_frequency)
            await ctx.info(update_msg)

        # Sleep between updates
        if i < update_frequency - 1:
            await asyncio.sleep(update_interval)

    end_time = datetime.utcnow()
    total_duration = (end_time - start_time).total_seconds()

    # Calculate total distance traveled
    distance = ((storm_lat - (storm_lat - 0.2 * update_frequency))**2 +
                (storm_lon - (storm_lon - 0.2 * update_frequency))**2) ** 0.5 * 69  # miles

    # Final classification
    if intensity < 39:
        category = "Tropical Depression"
    elif intensity < 74:
        category = "Tropical Storm"
    elif intensity < 96:
        category = "Category 1 Hurricane"
    elif intensity < 111:
        category = "Category 2 Hurricane"
    else:
        category = "Category 3 Hurricane"

    # Build final result
    result = f"Storm Tracking Complete\n"
    result += f"Starting Location: {location}\n"
    result += f"Duration: {total_duration:.1f} seconds\n"
    result += f"Final Classification: {category}\n"
    result += f"Final Intensity: {intensity} mph\n"
    result += f"Distance Traveled: {distance:.1f} miles\n\n"
    result += "Tracking History:\n"
    result += "\n".join(all_updates)
    result += f"\n\nTotal updates: {len(all_updates)}"

    return result


# =============================================================================
# =============================================================================
# Main - Run server with Streamable HTTP transport and API Key Auth
# =============================================================================
if __name__ == "__main__":
    import uvicorn
    from starlette.applications import Starlette
    from starlette.routing import Mount
    
    print("=" * 70)
    print("Weather MCP Server - FastMCP (Python) with API Key Auth")
    print("=" * 70)
    print(f"AUTHENTICATION: X-API-Key header required")
    print(f"API Key: {API_KEY}")
    print("")
    print("Protocol: MCP Streamable HTTP (spec 2025-06-18)")
    print("Endpoint: POST /mcp")
    print("")
    print("Tools (7 total - stateless + streaming):")
    print("")
    print("  Stateless Tools (quick JSON/SSE responses):")
    print("    1. get_weather         - Current weather data")
    print("    2. get_forecast        - Weather forecast (1-5 days)")
    print("    3. search_location     - Location search")
    print("    4. get_air_quality     - Air quality data")
    print("    5. get_weather_alerts  - Weather alerts")
    print("")
    print("  Streaming Tools (progressive SSE updates):")
    print("    6. monitor_weather     - Real-time weather monitoring (10-60s)")
    print("    7. track_storm         - Storm tracking with updates (20-120s)")
    print("")
    print("Starting server on port 8000...")
    print("=" * 70)

    # Get the FastMCP Starlette app and add middleware
    app = mcp.get_app(transport="streamable-http", path="/mcp")
    app.add_middleware(APIKeyMiddleware)
    
    uvicorn.run(app, host="0.0.0.0", port=8000)

