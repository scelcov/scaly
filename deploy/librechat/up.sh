#!/bin/sh
# Start LibreChat and the upscale container.
set -eu
cd "$(dirname "$0")"

mkdir -p images uploads logs skill data-node meili_data_v1.35.1

if [ ! -f .env ]; then
    cp .env.example .env
    uid=$(id -u)
    gid=$(id -g)
    # shellcheck disable=SC2016
    sed -i "s/^UID=.*/UID=${uid}/; s/^GID=.*/GID=${gid}/" .env
    secret() { openssl rand -hex "$1"; }
    sed -i \
        -e "s/^MEILI_MASTER_KEY=.*/MEILI_MASTER_KEY=$(secret 16)/" \
        -e "s/^JWT_SECRET=.*/JWT_SECRET=$(secret 32)/" \
        -e "s/^JWT_REFRESH_SECRET=.*/JWT_REFRESH_SECRET=$(secret 32)/" \
        -e "s/^ADMIN_PANEL_SESSION_SECRET=.*/ADMIN_PANEL_SESSION_SECRET=$(secret 32)/" \
        -e "s/^CREDS_KEY=.*/CREDS_KEY=$(secret 32)/" \
        -e "s/^CREDS_IV=.*/CREDS_IV=$(secret 16)/" \
        .env
fi

if ! grep -q '^UPSCALE_PUBLIC_URL=.\+' .env; then
    lan=$(ip -4 route get 1.1.1.1 2>/dev/null | awk '{
        for (i = 1; i <= NF; i++) if ($i == "src") { print $(i + 1); exit }
    }')
    if [ -n "${lan}" ]; then
        if grep -q '^UPSCALE_PUBLIC_URL=' .env; then
            sed -i "s|^UPSCALE_PUBLIC_URL=.*|UPSCALE_PUBLIC_URL=http://${lan}:8788|" .env
        else
            echo "UPSCALE_PUBLIC_URL=http://${lan}:8788" >> .env
        fi
    fi
fi

rm -f docker-compose.tunnel.yml
compose="docker compose"
if grep -q '^CLOUDFLARE_TUNNEL_TOKEN=.\+' .env; then
    cat > docker-compose.tunnel.yml <<'EOF'
services:
  cloudflared:
    command: tunnel --no-autoupdate run --token ${CLOUDFLARE_TUNNEL_TOKEN}
EOF
    compose="docker compose -f docker-compose.yml -f docker-compose.override.yml -f docker-compose.tunnel.yml"
fi

# shellcheck disable=SC2086
$compose up -d --build
echo "LibreChat: http://localhost:${PORT:-3080}"
grep '^UPSCALE_PUBLIC_URL=' .env

if ! grep -q '^CLOUDFLARE_TUNNEL_TOKEN=.\+' .env; then
    echo "Waiting for the HTTPS tunnel..."
    url=""
    i=0
    while [ "$i" -lt 40 ]; do
        url=$(docker logs cloudflared 2>&1 | grep -oE 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' | head -n 1)
        if [ -n "$url" ]; then
            break
        fi
        i=$((i + 1))
        sleep 1
    done
    if [ -n "$url" ]; then
        printf '%s\n' "$url" > chat-url.txt
        echo "iPhone: open ${url} in Safari, then Share → Add to Home Screen"
    else
        echo "Tunnel URL not ready yet. Run: docker logs cloudflared" >&2
    fi
fi
