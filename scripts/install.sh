#!/bin/bash
set -e

echo "OpenHack Installer"
echo "=================="

command -v docker >/dev/null 2>&1 || { echo "Docker is required but not installed."; exit 1; }
command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1 || { echo "Docker Compose v2 is required."; exit 1; }

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_DIR"

if [ ! -f ".env" ]; then
    echo "Generating configuration from .env.example..."
    cp .env.example .env

    JWT_SECRET=""
    OPENHACK_SECRET=""
    if command -v openssl >/dev/null 2>&1; then
        JWT_SECRET=$(openssl rand -hex 32)
        OPENHACK_SECRET=$(openssl rand -hex 32)
    elif command -v python3 >/dev/null 2>&1; then
        JWT_SECRET=$(python3 -c "import secrets; print(secrets.token_hex(32))")
        OPENHACK_SECRET=$(python3 -c "import secrets; print(secrets.token_hex(32))")
    fi

    if [ -n "$JWT_SECRET" ]; then
        if grep -q "JWT_SECRET=changeme" .env 2>/dev/null; then
            sed -i "s/JWT_SECRET=changeme/JWT_SECRET=$JWT_SECRET/" .env
        fi
    fi
    if [ -n "$OPENHACK_SECRET" ]; then
        if grep -q "OPENHACK_SECRET=changeme" .env 2>/dev/null; then
            sed -i "s/OPENHACK_SECRET=changeme/OPENHACK_SECRET=$OPENHACK_SECRET/" .env
        fi
    fi

    if [ -z "$JWT_SECRET" ] || [ -z "$OPENHACK_SECRET" ]; then
        echo "WARNING: Could not auto-generate secrets. Set JWT_SECRET and OPENHACK_SECRET in .env manually."
    fi
else
    echo ".env already exists, skipping configuration generation."
fi

echo "Building and starting services (core profile: auth, core, gateway)..."
docker compose up -d --build

echo ""
echo "Waiting for services to be healthy..."
MAX_WAIT=120
ELAPSED=0
ALL_HEALTHY=false
while [ $ELAPSED -lt $MAX_WAIT ]; do
    HEALTH=$(docker compose exec -T gateway-svc wget -qO- http://localhost:8000/health 2>/dev/null || echo "not_ready")
    if echo "$HEALTH" | grep -q "healthy"; then
        ALL_HEALTHY=true
        break
    fi
    sleep 5
    ELAPSED=$((ELAPSED + 5))
    echo "  Waiting... (${ELAPSED}s/${MAX_WAIT}s)"
done

echo ""
docker compose ps

if [ "$ALL_HEALTHY" = true ]; then
    echo ""
    echo "OpenHack is running!"
    echo "  Gateway:    http://localhost:8000"
    echo "  Health:     http://localhost:8000/health"
    echo ""
    echo "Quick start:"
    echo '  curl -X POST http://localhost:8000/api/auth/register \'
    echo '    -H "Content-Type: application/json" \'
    echo '    -d '"'"'{"email":"admin@test.com","password":"Password1!","name":"Admin"}'"'"''
    echo ""
    echo "Optional: start additional service profiles:"
    echo "  docker compose --profile judging up -d"
    echo "  docker compose --profile comms up -d"
    echo "  docker compose --profile ai up -d"
    echo "  docker compose --profile full up -d"
else
    echo ""
    echo "WARNING: Services did not become healthy within ${MAX_WAIT}s."
    echo "Check logs with: docker compose logs"
fi