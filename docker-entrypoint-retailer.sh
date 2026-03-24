#!/bin/sh
set -e
install -m 700 -o git -g git -d /home/git/.ssh
install -m 600 -o git -g git /ssh-keys/authorized_keys /home/git/.ssh/authorized_keys
exec /usr/sbin/sshd -D -e
