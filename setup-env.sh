#!/bin/bash

# 1.1. setup environment and then restart shell

apt update
apt -y install htop gcc g++ clang llvm libomp-dev make git mosh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 1.1.1. setup mosh-server
locale-gen "en_US.UTF-8"
update-locale LC_ALL="en_US.UTF-8"
echo LANG=en_US.UTF-8 > /etc/locale.conf
