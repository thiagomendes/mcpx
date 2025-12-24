FROM python:3.12-slim

WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy server code
COPY server.py .

# Expose MCP port
EXPOSE 8000

# Run stateless MCP server
CMD ["python", "server.py"]
