# Cursor Cloud base for herdr — Debian/Ubuntu for Computer Use / desktop.
# Do not COPY the project; Cursor checks out the selected revision separately.
#
# Goals: gh, Node (agent-browser), SSH (fleet MCP), jq/rg, sudo, and a desktop
# stack compatible with Cursor computer-use. Rust/Zig builds stay CI-only.

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        wget \
        git \
        jq \
        ripgrep \
        openssh-client \
        sudo \
        gnupg \
        unzip \
        python3 \
        python3-pip \
        build-essential \
        # Desktop / computer-use helpers (Cursor also may layer its own VNC)
        xfce4 \
        xfce4-terminal \
        dbus-x11 \
        tigervnc-standalone-server \
        tigervnc-common \
        novnc \
        websockify \
        # Browser engine runtime deps (Lightpanda is downloaded in install)
        libstdc++6 \
        libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

# GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
      | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
      > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && rm -rf /var/lib/apt/lists/*

# Node 22 (agent-browser + vault MCP builds)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Tailscale package (daemon started in userspace mode from start script)
RUN curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/noble.noarmor.gpg \
      | tee /usr/share/keyrings/tailscale-archive-keyring.gpg >/dev/null \
    && curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/noble.tailscale-keyring.list \
      | tee /etc/apt/sources.list.d/tailscale.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends tailscale \
    && rm -rf /var/lib/apt/lists/*

# Non-root user Cursor Cloud expects
RUN id -u ubuntu >/dev/null 2>&1 || useradd -m -s /bin/bash ubuntu \
    && usermod -aG sudo ubuntu \
    && echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ubuntu \
    && chmod 0440 /etc/sudoers.d/ubuntu

# PATH helpers for user-installed CLIs
ENV PATH="/home/ubuntu/.local/bin:${PATH}"

USER ubuntu
WORKDIR /home/ubuntu

HEALTHCHECK NONE
