#!/bin/sh
set -e
mkdir -p /root/.ssh
install -m 600 /ssh-keys/id_ed25519 /root/.ssh/id_ed25519
exec git-retail up
