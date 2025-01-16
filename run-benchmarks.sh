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
# apt -y install htop gcc g++ clang make git mosh golang libgmp-dev llvm python3-full
# python3 -m venv ./venv
# ./venv/bin/pip install aiogram
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

cores_count=("1" "2" "4" "8" "16")
taskset_cpu_list=("0" "0-1" "0-3" "0-7" "0-15")
jobs_count=${#cores_count[@]}

print_cores_msg () {
  printf "\n\n\n********** %s **********\n\n\n" "$1" >> "$2"
}

print_msg () {
  printf "\n\n\n~~~~~~~~~~ %s ~~~~~~~~~~\n\n\n" "$1" >> "$2"
}

lscpu | grep "Model\ name" | head -n 1 >> "$paste_file"
lscpu | grep "CPU(s)"      | head -n 1 >> "$paste_file"

# 2. prepare benchmarks

# 2.1. prepare rust-kzg
git clone https://github.com/sifraitech/rust-kzg
cd rust-kzg || exit

# 2.2. prepare rust-kzg with blst backend and c-kzg-4844
cd blst || exit
cargo rustc --release --crate-type=staticlib --features=parallel
mv ../target/release/librust_kzg_blst.a ../target/release/rust_kzg_blst.a
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit
git -c advice.detachedHead=false checkout 1bccee0878ffc80efe8741afdb5793ef9105aa35 # TODO: keep this updated
git submodule update --init
cd src || exit
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS
cd ..
git apply < ../rust.patch || exit
git apply < ../go.patch || exit
cd ../..

# 2.3. prepare rust-kzg with mcl backend
cd mcl/kzg && bash build.sh
cd ../../..

# 2.4. prepare c-kzg-4844
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844/src || exit
make all
cd ../..

# 2.5. prepare go-kzg-4844
git clone https://github.com/crate-crypto/go-kzg-4844

# 2.6. prepare constantine
git clone https://github.com/mratsim/constantine

for (( i=0; i<jobs_count; i++ ));
do
  # 3. run benchmarks

  print_cores_msg "BENCHMARKING ON ${cores_count[$i]} CORES" "$paste_file"

  # 3.1. go-kzg-4844
  cd go-kzg-4844 || exit
  print_msg "go-kzg-4844" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" go test -bench=. >> ../"$paste_file"
  cd ..

  # 3.2. rust binding (c-kzg-4844)
  cd c-kzg-4844 || exit
  print_msg "rust binding (c-kzg-4844)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench >> ../"$paste_file"
  cd ..
  
  # 3.3. constantine
  cd constantine || exit
  print_msg "constantine" ../"$paste_file"
  CC=clang taskset --cpu-list "${taskset_cpu_list[$i]}" nimble bench_eth_eip4844_kzg >> ../"$paste_file"
  cd ..

  # rust crates
  cd rust-kzg || exit

  # 3.3. rust-kzg with arkworks4 backend (sequential)
  print_msg "rust-kzg with arkworks4 backend (sequential)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand >> ../"$paste_file"

  print_msg "rust-kzg with arkworks4 backend (sequential, arkmsm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,arkmsm >> ../"$paste_file"

  print_msg "rust-kzg with arkworks4 backend (sequential, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,bgmw >> ../"$paste_file"

  # 3.4. rust-kzg with arkworks4 backend (parallel)
  print_msg "rust-kzg with arkworks4 backend (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,parallel >> ../"$paste_file"

  print_msg "rust-kzg with arkworks4 backend (parallel, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

  # 3.5. rust-kzg with zkcrypto backend (sequential)
  print_msg "rust-kzg with zkcrypto backend (sequential)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml >> ../"$paste_file"

  # 3.6. rust-kzg with zkcrypto backend (parallel)
  print_msg "rust-kzg with zkcrypto backend (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml --features parallel >> ../"$paste_file"

  # 3.7. rust-kzg with blst backend (sequential)
  print_msg "rust-kzg with blst backend (sequential)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst/Cargo.toml --no-default-features --features std,rand >> ../"$paste_file"

  print_msg "rust-kzg with blst backend (sequential, arkmsm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst/Cargo.toml --no-default-features --features std,rand,arkmsm >> ../"$paste_file"

  print_msg "rust-kzg with blst backend (sequential, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst/Cargo.toml --no-default-features --features std,rand,bgmw >> ../"$paste_file"

  # 3.8. rust-kzg with blst backend (parallel)
  print_msg "rust-kzg with blst backend (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst/Cargo.toml --no-default-features --features std,rand,parallel >> ../"$paste_file"

  print_msg "rust-kzg with blst backend (parallel, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst/Cargo.toml --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

  # 3.9. rust-kzg with mcl backend (sequential) FIXME: mcl benchmarks do not work
  #print_msg "rust-kzg with mcl backend (sequential)" ../"$paste_file"
  #taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl/kzg-bench/Cargo.toml >> ../"$paste_file"

  # 3.10. rust-kzg with mcl backend (parallel)
  #print_msg "rust-kzg with mcl backend (parallel)" ../"$paste_file"
  #taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl/kzg-bench/Cargo.toml --features rust-kzg-mcl/parallel >> ../"$paste_file"

  # 3.11. rust-kzg with constantine backend (sequential)
  print_msg "rust-kzg with constantine backend (sequential)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand >> ../"$paste_file"

  print_msg "rust-kzg with constantine backend (sequential, arkmsm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,arkmsm >> ../"$paste_file"

  print_msg "rust-kzg with constantine backend (sequential, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,bgmw >> ../"$paste_file"

  # 3.12. rust-kzg with constantine backend (sequential, constantine_msm)
  print_msg "rust-kzg with constantine backend (sequential, constantine_msm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,constantine_msm >> ../"$paste_file"

  # 3.13. rust-kzg with constantine backend (parallel)
  print_msg "rust-kzg with constantine backend (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,parallel >> ../"$paste_file"

  print_msg "rust-kzg with constantine backend (parallel, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

  # 3.14. rust-kzg with constantine backend (parallel, constantine_msm)
  print_msg "rust-kzg with constantine backend (parallel, constantine_msm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path constantine/Cargo.toml --no-default-features --features std,rand,parallel,constantine_msm >> ../"$paste_file"

  # 3.15. rust-kzg with arkworks4 backend (sequential)
  print_msg "rust-kzg with arkworks4 backend (sequential)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand >> ../"$paste_file"

  print_msg "rust-kzg with arkworks4 backend (sequential, arkmsm)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,arkmsm >> ../"$paste_file"

  print_msg "rust-kzg with arkworks4 backend (sequential, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks4/Cargo.toml --no-default-features --features std,rand,bgmw >> ../"$paste_file"

  # 3.16. rust-kzg with arkworks5 backend (parallel)
  print_msg "rust-kzg with arkworks5 backend (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks5/Cargo.toml --no-default-features --features std,rand,parallel >> ../"$paste_file"

  print_msg "rust-kzg with arkworks5 backend (parallel, bgmw)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path arkworks5/Cargo.toml --no-default-features --features std,rand,parallel,bgmw >> ../"$paste_file"

  # 3.17. rust binding (rust-kzg with blst backend)
  print_msg "rust binding (rust-kzg with blst backend)" ../"$paste_file"
  cd blst/c-kzg-4844/bindings/rust/ || exit
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench >> ../../../../../"$paste_file"
  cd ../../../..

  # 3.18. go binding (rust-kzg with blst backend)
  print_msg "go binding (rust-kzg with blst backend)" ../"$paste_file"
  cd blst/c-kzg-4844/bindings/go/ || exit
  export CGO_CFLAGS="-O2 -D__BLST_PORTABLE__"
  taskset --cpu-list "${taskset_cpu_list[$i]}" go test -run ^$ -bench . >> ../../../../../"$paste_file"
  unset CGO_CFLAGS
  cd ../../../..

  cd ..
done

# 4. collect results

paste_url=$(curl --upload-file "$paste_file" "https://paste.c-net.org/")

echo "=========================================="
echo "Uploaded to $paste_url"
echo "=========================================="
echo "Script finished in $(printf '%02dh:%02dm:%02ds\n' $((SECONDS/3600)) $((SECONDS%3600/60)) $((SECONDS%60)))"
echo "Success!"
