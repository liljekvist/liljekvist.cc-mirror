#!/usr/bin/env bash
# =============================================================================
# setup-vps.sh — Deploy setup for liljekvist-cc-mainpage
#
# Assumptions (already done on this server):
#   ✔ Docker + Docker Compose plugin installed
#   ✔ TLS certificate issued by Certbot for liljekvist.cc
#   ✔ DNS pointing to this server
#   ✔ Firewall configured
#
# This script does exactly four things:
#   1. Create a dedicated "deploy" user with docker group access
#   2. Generate an Ed25519 SSH deploy key and install the public half
#   3. Create the deploy directory and write docker-compose.yml into it
#   4. Replace the existing static nginx config with a reverse-proxy config
#      (keeps all existing SSL / ACME / security-header blocks intact)
#
# Usage:
#   chmod +x setup-vps.sh
#   sudo ./setup-vps.sh
# =============================================================================

set -euo pipefail

# ── Configuration — edit these before running ─────────────────────────────────
DEPLOY_USER="deploy"
DEPLOY_DIR="/srv/liljekvist-cc-mainpage"
DOMAIN="liljekvist.cc"
REGISTRY_IMAGE="registry.gitlab.com/liljekvist/liljekvist-cc-mainpage"
KEY_COMMENT="gitlab-ci@${DOMAIN}"
NGINX_CONF="/etc/nginx/sites-available/${DOMAIN}.conf"

# Colours
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${GREEN}[✔]${RESET} $*"; }
warn()    { echo -e "${YELLOW}[!]${RESET} $*"; }
section() { echo -e "\n${CYAN}${BOLD}── $* ──${RESET}"; }
die()     { echo -e "${RED}[✘] $*${RESET}" >&2; exit 1; }

# ── Guards ────────────────────────────────────────────────────────────────────
[[ $EUID -eq 0 ]] || die "Please run as root: sudo ./setup-vps.sh"
command -v docker &>/dev/null || die "Docker not found. Install Docker first."
docker compose version &>/dev/null || die "Docker Compose plugin not found."
info "Docker: $(docker --version)"
info "Compose: $(docker compose version)"

# =============================================================================
# 1. Create deploy user
# =============================================================================
section "Creating deploy user: ${DEPLOY_USER}"

if id "${DEPLOY_USER}" &>/dev/null; then
    warn "User '${DEPLOY_USER}' already exists — skipping creation"
else
    useradd -m -s /bin/bash "${DEPLOY_USER}"
    info "User '${DEPLOY_USER}' created"
fi

usermod -aG docker "${DEPLOY_USER}"
info "Added '${DEPLOY_USER}' to the docker group"

# =============================================================================
# 2. Generate SSH deploy key
# =============================================================================
section "Generating SSH deploy key"

DEPLOY_HOME=$(getent passwd "${DEPLOY_USER}" | cut -d: -f6)
SSH_DIR="${DEPLOY_HOME}/.ssh"
KEY_PATH="${SSH_DIR}/id_ed25519_deploy"

# Create .ssh directory with correct ownership before switching user.
mkdir -p "${SSH_DIR}"
chown "${DEPLOY_USER}:${DEPLOY_USER}" "${SSH_DIR}"
chmod 700 "${SSH_DIR}"

# Generate the key AS the deploy user so ownership and permissions are correct
# from the start — no root-owned files in the user's home directory.
runuser -u "${DEPLOY_USER}" -- \
    ssh-keygen -t ed25519 -C "${KEY_COMMENT}" -f "${KEY_PATH}" -N ""

# Register the public key as an authorised key for the deploy user.
runuser -u "${DEPLOY_USER}" -- \
    bash -c "cat '${KEY_PATH}.pub' >> '${SSH_DIR}/authorized_keys'"
chmod 600 "${SSH_DIR}/authorized_keys"
chown "${DEPLOY_USER}:${DEPLOY_USER}" "${SSH_DIR}/authorized_keys"

info "Deploy key generated at ${KEY_PATH} (owned by ${DEPLOY_USER})"
info "Public key installed into ${SSH_DIR}/authorized_keys"

# =============================================================================
# 3. Create deploy directory + docker-compose.yml
# =============================================================================
section "Creating deploy directory: ${DEPLOY_DIR}"

mkdir -p "${DEPLOY_DIR}"
chown "${DEPLOY_USER}:${DEPLOY_USER}" "${DEPLOY_DIR}"

cat > "${DEPLOY_DIR}/docker-compose.yml" << COMPOSE
services:
  web:
    image: \${REGISTRY_IMAGE:-${REGISTRY_IMAGE}}:\${IMAGE_TAG:-latest}
    container_name: liljekvist-cc-mainpage
    restart: unless-stopped
    ports:
      - "127.0.0.1:3000:3000"
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 128M
    healthcheck:
      test: ["CMD-SHELL", "curl -sf http://localhost:3000/ || exit 1"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s
COMPOSE

chown "${DEPLOY_USER}:${DEPLOY_USER}" "${DEPLOY_DIR}/docker-compose.yml"
info "docker-compose.yml written to ${DEPLOY_DIR}"

# =============================================================================
# 4. Replace static nginx config with reverse-proxy config
#    Keeps all existing SSL / ACME / certbot-managed blocks.
#    Creates a timestamped backup of the original file first.
# =============================================================================
section "Updating nginx config: ${NGINX_CONF}"

# Detect the config file — try the sites-available path, then conf.d.
if [[ ! -f "${NGINX_CONF}" ]]; then
    ALT="/etc/nginx/conf.d/${DOMAIN}.conf"
    if [[ -f "${ALT}" ]]; then
        NGINX_CONF="${ALT}"
        warn "Using conf.d path: ${NGINX_CONF}"
    else
        die "Cannot find nginx config for ${DOMAIN}. Looked at:
  ${NGINX_CONF}
  ${ALT}
Edit NGINX_CONF at the top of this script and re-run."
    fi
fi

BACKUP="${NGINX_CONF}.bak.$(date +%Y%m%d%H%M%S)"
cp "${NGINX_CONF}" "${BACKUP}"
info "Original config backed up to ${BACKUP}"

# Write the new config — identical SSL/ACME structure, static serving replaced
# with proxy_pass to 127.0.0.1:3000, /ws block added for WebSocket support.
cat > "${NGINX_CONF}" << 'NGINX'
###############################################
# HTTP (port 80) - ACME + redirect to HTTPS
###############################################
server {
    listen 80;
    listen [::]:80;
    server_name liljekvist.cc www.liljekvist.cc;

    # ACME Challenge
    location ^~ /.well-known/acme-challenge/ {
        root /var/www/_letsencrypt;
        try_files $uri =404;
    }

    # Redirect everything else to HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}

###############################################
# HTTPS (port 443) - Reverse proxy to Axum
###############################################
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name liljekvist.cc www.liljekvist.cc;

    # SSL - Managed by Certbot (unchanged)
    ssl_certificate     /etc/letsencrypt/live/liljekvist.cc/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/liljekvist.cc/privkey.pem;
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam         /etc/letsencrypt/ssl-dhparams.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options SAMEORIGIN always;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Permissions-Policy "interest-cohort=()" always;

    # ACME reachable on HTTPS
    location ^~ /.well-known/acme-challenge/ {
        root /var/www/_letsencrypt;
        try_files $uri =404;
    }

    # ── WebSocket (/ws) ───────────────────────────────────────────────────
    # Must be listed before the generic location / block.
    location /ws {
        proxy_pass          http://127.0.0.1:3000;
        proxy_http_version  1.1;
        proxy_set_header    Upgrade           $http_upgrade;
        proxy_set_header    Connection        "upgrade";
        proxy_set_header    Host              $host;
        proxy_set_header    X-Real-IP         $remote_addr;
        proxy_set_header    X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header    X-Forwarded-Proto $scheme;
        proxy_read_timeout  3600s;
        proxy_send_timeout  3600s;
    }

    # ── HTTP reverse proxy ────────────────────────────────────────────────
    location / {
        proxy_pass            http://127.0.0.1:3000;
        proxy_http_version    1.1;
        proxy_set_header      Connection        "";
        proxy_set_header      Host              $host;
        proxy_set_header      X-Real-IP         $remote_addr;
        proxy_set_header      X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header      X-Forwarded-Proto $scheme;
        proxy_connect_timeout 5s;
        proxy_read_timeout    30s;
        proxy_send_timeout    30s;
        proxy_buffering       on;
        proxy_buffer_size     8k;
        proxy_buffers         16 8k;
    }
}
NGINX

nginx -t || {
    warn "nginx config test failed — restoring backup"
    cp "${BACKUP}" "${NGINX_CONF}"
    nginx -t
    die "Restored original config. Fix the error above and re-run."
}

systemctl reload nginx
info "nginx reloaded with new reverse-proxy config"

# =============================================================================
# 5. Summary
# =============================================================================
section "Setup complete — action required"

PRIVATE_KEY=$(cat "${KEY_PATH}")
PUBLIC_KEY=$(cat "${KEY_PATH}.pub")

echo ""
echo -e "${BOLD}${YELLOW}┌──────────────────────────────────────────────────────────────────────┐${RESET}"
echo -e "${BOLD}${YELLOW}│  PRIVATE KEY — paste into GitLab: Settings → CI/CD → Variables       │${RESET}"
echo -e "${BOLD}${YELLOW}│  Name: DEPLOY_KEY  │  Type: File  │  Protected: ✔  │  Masked: ✔      │${RESET}"
echo -e "${BOLD}${YELLOW}└──────────────────────────────────────────────────────────────────────┘${RESET}"
echo ""
echo "${PRIVATE_KEY}"
echo ""
echo -e "${BOLD}${CYAN}┌──────────────────────────────────────────────────────────────────────┐${RESET}"
echo -e "${BOLD}${CYAN}│  PUBLIC KEY (installed in authorized_keys — kept here for reference)  │${RESET}"
echo -e "${BOLD}${CYAN}└──────────────────────────────────────────────────────────────────────┘${RESET}"
echo ""
echo "${PUBLIC_KEY}"
echo ""
# Note: the private key remains at ${KEY_PATH} owned by ${DEPLOY_USER}.
# It is only needed by GitLab CI — if you prefer to remove it from the
# server after copying it, run: sudo rm "${KEY_PATH}"

echo -e "${BOLD}Set these four variables in GitLab (Settings → CI/CD → Variables):${RESET}"
echo ""
printf "  ${CYAN}%-20s${RESET} %s\n" "DEPLOY_HOST"  "$(hostname -I | awk '{print $1}')  ← or use ${DOMAIN}"
printf "  ${CYAN}%-20s${RESET} %s\n" "DEPLOY_USER"  "${DEPLOY_USER}"
printf "  ${CYAN}%-20s${RESET} %s\n" "DEPLOY_KEY"   "<paste the private key above — Type: File>"
printf "  ${CYAN}%-20s${RESET} %s\n" "DEPLOY_DIR"   "${DEPLOY_DIR}"
echo ""
info "All done. Push to main/master to trigger the first deployment."





