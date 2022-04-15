#!/bin/bash
# shellcheck disable=SC2103,SC2002

SECONDS=0

paste_name="linode_debian_eu_central_cpu_$(nproc)"
paste_file="$paste_name.txt"

omp_thread_count=("1" "2" "4" "8" "16")
taskset_cpu_list=("0" "0-1" "0-3" "0-7" "0-15")
jobs_count=${#omp_thread_count[@]}

# 1.2. post shell restart

rustup install 1.58.1
rustup default 1.58.1

print_thread_msg () {
  printf "\n\n\n********** %s **********\n\n\n" "$1" >> "$2"
}

print_msg () {
  printf "\n\n\n~~~~~~~~~~ %s ~~~~~~~~~~\n\n\n" "$1" >> "$2"
}

lscpu | grep "Model\ name" | head -n 1 >> "$paste_file"
lscpu | grep "CPU(s)"      | head -n 1 >> "$paste_file"

# 2. run individual benches

# 2.1. c-kzg [prep-up]

git clone https://github.com/supranational/blst.git
cd blst || exit
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
bash build.sh
unset CFLAGS
cd ..
git clone https://github.com/tesa4436/c-kzg.git
cp -r blst/* c-kzg/lib/
cp -r blst/bindings/*.h c-kzg/inc/

# 2.2. kzg [prep-up]

git clone --single-branch --branch research https://github.com/sifraitech/kzg
cd kzg || exit

# 2.2.1. prep up ckzg
export "LIBOMP_PATH=$(find /usr/lib/llvm* -name libiomp5.so | head -n 1)"
export "RUSTFLAGS=-C link-arg=""$LIBOMP_PATH"""
cd ckzg && bash build.sh
cd ..

# 2.2.2. prep up mcl-kzg10-rust
cd mcl-kzg/kzg && bash build.sh
cd ../../..

for (( i=0; i<jobs_count; i++ ));
do
  print_thread_msg "BENCHMARKING ON ${omp_thread_count[$i]} THREADS" ../../"$paste_file"
  export OMP_NUM_THREADS="${omp_thread_count[$i]}"

  # 2.3.1. c-kzg [original]

  print_msg "c-kzg [original]" ../../"$paste_file"
  cd c-kzg/src || exit
  git restore Makefile
  git checkout main
  taskset --cpu-list "${taskset_cpu_list[$i]}" make bench >> ../../"$paste_file"

  # 2.3.2. c-kzg [parallelized]

  print_msg "c-kzg [parallelized]" ../../"$paste_file"
  git checkout openmp
  eval "$(sed -i "s/KZG_CFLAGS =/KZG_CFLAGS = -fPIE -fopenmp/" Makefile)"
  eval "$(sed -i 's/KZG_CFLAGS += -O/KZG_CFLAGS += -Ofast/' Makefile)"
  taskset --cpu-list "${taskset_cpu_list[$i]}" make bench >> ../../"$paste_file"
  cd ../..

  # rust crates
  cd kzg || exit

  # 2.3.1. arkworks

  print_msg "arkworks [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml >> ../"$paste_file"
  print_msg "arkworks [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.2. zkcrypto

  print_msg "zkcrypto [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml >> ../"$paste_file"
  print_msg "zkcrypto [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.3. blst-from-scratch

  print_msg "blst-from-scratch [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml >> ../"$paste_file"
  print_msg "blst-from-scratch [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.4. ckzg

  print_msg "ckzg [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path ckzg/Cargo.toml >> ../"$paste_file"
  print_msg "ckzg [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path ckzg/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.5. mcl-kzg10-rust

  print_msg "mcl-kzg10-rust [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml >> ../"$paste_file"
  print_msg "mcl-kzg10-rust [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml --features mcl_rust/parallel >> ../"$paste_file"

  cd ..
  unset OMP_NUM_THREADS
done

# 3. collect results

paste_url=$(
  curl -d "api_paste_code=$(cat "$paste_file")" \
       -d "api_paste_private=1" \
       -d "api_paste_name=$paste_name" \
       -d "api_dev_key=ba2fd41ca7923844193bf05d3b19ed32" \
       -d "api_option=paste" "https://pastebin.com/api/api_post.php"
)

echo "=========================================="
echo "Uploaded to $paste_url"
echo "=========================================="
echo "Script finished in $(printf '%02dh:%02dm:%02ds\n' $((SECONDS/3600)) $((SECONDS%3600/60)) $((SECONDS%60)))"
echo "Success!"
