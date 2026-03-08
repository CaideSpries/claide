#!/bin/sh
set -e

# Fix permissions on data directory if running as root
if [ "$(id -u)" = '0' ]; then
    # Ensure data directory exists and has correct ownership
    mkdir -p /data
    chown -R claide:claide /data

    # Drop to claide user and execute command
    exec gosu claide "$@"
fi

exec "$@"
