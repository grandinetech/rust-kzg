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
# apt -y install htop gcc g++ clang make git mosh golang default-jdk unzip
# curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
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

# 1.4. setup gradle and environment variables
cd /tmp || exit
curl -O https://downloads.gradle-dn.com/distributions/gradle-7.6-bin.zip
unzip gradle-7.6-bin.zip
mkdir /opt/gradle
cp -pr gradle-7.6/* /opt/gradle
cd ~ || exit
touch /etc/profile.d/java_home.sh
echo "export JAVA_HOME=\"/usr/lib/jvm/default-java\"" >> /etc/profile.d/java_home.sh
echo "export PATH=$JAVA_HOME/bin:$PATH" >> /etc/profile.d/java_home.sh
echo "export PATH=/opt/gradle/bin:${PATH}" | tee /etc/profile.d/gradle.sh
source /etc/profile.d/java_home.sh
source /etc/profile.d/gradle.sh

# 2. prepare benchmarks

# 2.1. prepare rust-kzg
git clone https://github.com/sifraitech/rust-kzg
cd rust-kzg || exit

# 2.2. prepare blst-from-scratch
cd blst-from-scratch || exit
cargo rustc --release --crate-type=staticlib --features=parallel
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844 || exit 1
git -c advice.detachedHead=false checkout fd24cf8e1e2f09a96b4e62a595b4e49f046ce6cf # TODO: keep this updated
git submodule update --init
cd src || exit
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
make blst
unset CFLAGS
cd ..
git apply < ../rust.patch
git apply < ../java.patch
git apply < ../go.patch
cd ../..

# 2.3. prepare mcl
cd mcl/kzg && bash build.sh
cd ../../..

# 2.4. prepare c-kzg-4844
git clone https://github.com/ethereum/c-kzg-4844.git
cd c-kzg-4844/src || exit 1
make all
cd ../..

# 2.5. prepare go-kzg-4844
git clone https://github.com/crate-crypto/go-kzg-4844

for (( i=0; i<jobs_count; i++ ));
do
  # 3. run benchmarks

  print_cores_msg "BENCHMARKING ON ${cores_count[$i]} CORES" "$paste_file"

  # 3.1. go-kzg-4844
  cd go-kzg-4844 || exit
  print_msg "go-kzg-4844" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" go test -bench=. >> ../"$paste_file"
  cd ..

  # 3.2. c-kzg-4844 (java binding)
  cd c-kzg-4844/bindings/java || exit
  print_msg "c-kzg-4844 (java binding)" ../../../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" make build benchmark >> ../../../"$paste_file"
  cd ../../..

  # rust crates
  cd rust-kzg || exit

  # 3.3. arkworks (original)
  print_msg "arkworks (original)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml >> ../"$paste_file"

  # 3.4. arkworks (parallel)
  print_msg "arkworks (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml --features parallel >> ../"$paste_file"

  # 3.5. zkcrypto (original)
  print_msg "zkcrypto (original)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml >> ../"$paste_file"

  # 3.6. zkcrypto (parallel)
  print_msg "zkcrypto (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml --features parallel >> ../"$paste_file"

  # 3.7. blst-from-scratch (original)
  print_msg "blst-from-scratch (original)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml >> ../"$paste_file"

  # 3.8. blst-from-scratch (parallel)
  print_msg "blst-from-scratch (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml --features parallel >> ../"$paste_file"

  # 3.9. mcl (original)
  print_msg "mcl (original)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl/kzg-bench/Cargo.toml >> ../"$paste_file"

  # 3.10. mcl (parallel)
  print_msg "mcl (parallel)" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl/kzg-bench/Cargo.toml --features mcl_rust/parallel >> ../"$paste_file"

  # 3.11. blst-from-scratch (rust binding)
  print_msg "blst-from-scratch (rust binding)" ../"$paste_file"
  cd blst-from-scratch/c-kzg-4844/bindings/rust/ || exit 1
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench >> ../../../../../"$paste_file"
  cd ../../../..

  # 3.12. blst-from-scratch (java binding)
  print_msg "blst-from-scratch (java binding)" ../"$paste_file"
  cd blst-from-scratch/c-kzg-4844/bindings/java/ || exit 1
  taskset --cpu-list "${taskset_cpu_list[$i]}" make build benchmark >> ../../../../../"$paste_file"
  cd ../../../..

  # 3.13. blst-from-scratch (go binding)
  print_msg "blst-from-scratch (go binding)" ../"$paste_file"
  cd blst-from-scratch/c-kzg-4844/bindings/go/ || exit 1
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
