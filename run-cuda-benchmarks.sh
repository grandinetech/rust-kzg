#!/bin/bash
# shellcheck disable=SC2103,SC2002
# shellcheck source=/dev/null

# 1.1. Follow the instructions in the comments, then exit and open a connection through the Mosh shell.
#
# IMPORTANT!!!
#
# Switch to the Debian Sid branch by modifying the file "/etc/apt/sources.list", and then upgrade all packages.
# This process will involve updating grub and selecting options on the screen.
# It is necessary because the go binding will not work with outdated libc family packages,
# and the current version of Go is too low to run benchmarks from "go-kzg-4844".
#
# This script is designed to be run once and forgotten about.
#
# 1.2. setup system
# apt -y install htop gcc g++ clang make git mosh golang libgmp-dev llvm python3 python3-pip
# pip install aiogram
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
# curl https://nim-lang.org/choosenim/init.sh -sSf | sh
# choosenim 1.6.14
#
# 1.3. setup mosh-server
# locale-gen en_US.UTF-8
# update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
# source /etc/default/locale

SECONDS=0

paste_name="linode_benchmarks"
paste_file="$paste_name.txt"

print_msg () {
  printf "\n\n\n~~~~~~~~~~ %s ~~~~~~~~~~\n\n\n" "$1" >> "$2"
}

lscpu >> "$paste_file"
nvidia-smi >> "$paste_file"

# 2. prepare benchmarks

# 2.1. prepare rust-kzg
git clone https://github.com/ArtiomTr/rust-kzg.git
cd rust-kzg || exit
git checkout Integrate_sppark_msm || exit

# 3. run benchmarks
print_msg "rust-kzg with blst backend (parallel, bgmw)" ../"$paste_file"
cargo bench --manifest-path blst/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"
cargo bench --manifest-path blst/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

print_msg "rust-kzg with blst backend (sppark)" ../"$paste_file"
cargo bench --manifest-path blst/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,sppark >> ../"$paste_file"
cargo bench --manifest-path blst/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,sppark >> ../"$paste_file"

print_msg "rust-kzg with arkworks backend (parallel, bgmw)" ../"$paste_file"
cargo bench --manifest-path arkworks/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"
cargo bench --manifest-path arkworks/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,bgmw 

print_msg "rust-kzg with arkworks3 backend (parallel, bgmw)" ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel >> ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel >> ../"$paste_file"

print_msg "rust-kzg with arkworks3 backend (sppark)" ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,sppark >> ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,sppark >> ../"$paste_file"

print_msg "rust-kzg with arkworks3 backend (sppark_wlc)" ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,sppark_wlc >> ../"$paste_file"
cargo bench --manifest-path arkworks3/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,sppark_wlc >> ../"$paste_file"

print_msg "rust-kzg with constantine backend (parallel, bgmw)" ../"$paste_file"
cargo bench --manifest-path constantine/Cargo.toml --bench eip_4844 --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"
cargo bench --manifest-path constantine/Cargo.toml --bench lincomb --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

cd ..

# 4. collect results

paste_url=$(curl --upload-file "$paste_file" "https://paste.c-net.org/")

echo "=========================================="
echo "Uploaded to $paste_url"
echo "=========================================="
echo "Script finished in $(printf '%02dh:%02dm:%02ds\n' $((SECONDS/3600)) $((SECONDS%3600/60)) $((SECONDS%60)))"
echo "Success!"
