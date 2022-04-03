#!/bin/bash

# 1.1. setup environment and then restart shell

apt update
apt -y install htop neofetch gcc g++ clang llvm libomp-dev make git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
