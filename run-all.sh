#!/bin/bash
# shellcheck disable=SC2103,SC2002

SECONDS=0

paste_name="linode_debian_eu_central_cpu_$(nproc)"
paste_file="$paste_name.txt"

omp_thread_count=("1" "2" "4" "8" "16")
taskset_cpu_list=("0" "0-1" "0-3" "0-7" "0-15")
jobs_count=${#omp_thread_count[@]}

c_kzg_fft_fr=()
c_kzg_fft_g1=()
c_kzg_recover=()
c_kzg_zero_poly=()
c_kzg_commit_to_poly=()
c_kzg_new_poly_div=()
c_kzg_das_extension=()
c_kzg_fk_single_da=()
c_kzg_fk_multi_da=()

# 1.2. post shell restart

rustup install 1.58.1
rustup default 1.58.1

print_thread_msg () {
  printf "\n\n\n********** %s **********\n\n\n" "$1" >> "$2"
}

print_msg () {
  printf "\n\n\n~~~~~~~~~~ %s ~~~~~~~~~~\n\n\n" "$1" >> "$2"
}

output_c_kzg () {
  { read -r min; } < <(printf "%s\n" "$2" | sort -n)
  echo "$1 $min ns/op" >> ../../"$paste_file"
}

bench_c_kzg () {
  for (( j=0; j<jobs_count; j++ ));
  do
    make clean
    bench_results="$(taskset --cpu-list "${taskset_cpu_list[$i]}" make bench)"
    # append results to arrays
    mapfile -t -O "${#c_kzg_fft_fr[@]}"         c_kzg_fft_fr         < <(echo "$bench_results" | grep 'fft_fr/scale_15'         | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_fft_g1[@]}"         c_kzg_fft_g1         < <(echo "$bench_results" | grep 'fft_g1/scale_15'         | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_recover[@]}"        c_kzg_recover        < <(echo "$bench_results" | grep 'recover/scale_15'        | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_zero_poly[@]}"      c_kzg_zero_poly      < <(echo "$bench_results" | grep 'zero_poly/scale_15'      | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_commit_to_poly[@]}" c_kzg_commit_to_poly < <(echo "$bench_results" | grep 'commit_to_poly/scale_15' | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_new_poly_div[@]}"   c_kzg_new_poly_div   < <(echo "$bench_results" | grep 'new_poly_div/scale_15'   | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_das_extension[@]}"  c_kzg_das_extension  < <(echo "$bench_results" | grep 'das_extension/scale_15'  | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_fk_single_da[@]}"   c_kzg_fk_single_da   < <(echo "$bench_results" | grep 'fk_single_da/scale_14'   | cut -d ' ' -f 2)
    mapfile -t -O "${#c_kzg_fk_multi_da[@]}"    c_kzg_fk_multi_da    < <(echo "$bench_results" | grep 'fk_multi_da/scale_14'    | cut -d ' ' -f 2)
  done

  output_c_kzg "fft_fr/scale_15"         "${c_kzg_fft_fr[@]}"
  output_c_kzg "fft_g1/scale_15"         "${c_kzg_fft_g1[@]}"
  output_c_kzg "recover/scale_15"        "${c_kzg_recover[@]}"
  output_c_kzg "zero_poly/scale_15"      "${c_kzg_zero_poly[@]}"
  output_c_kzg "commit_to_poly/scale_15" "${c_kzg_commit_to_poly[@]}"
  output_c_kzg "new_poly_div/scale_15"   "${c_kzg_new_poly_div[@]}"
  output_c_kzg "das_extension/scale_15"  "${c_kzg_das_extension[@]}"
  output_c_kzg "fk_single_da/scale_14"   "${c_kzg_fk_single_da[@]}"
  output_c_kzg "fk_multi_da/scale_14"    "${c_kzg_fk_multi_da[@]}"

  unset c_kzg_fft_fr
  unset c_kzg_fft_g1
  unset c_kzg_recover
  unset c_kzg_zero_poly
  unset c_kzg_commit_to_poly
  unset c_kzg_new_poly_div
  unset c_kzg_das_extension
  unset c_kzg_fk_single_da
  unset c_kzg_fk_multi_da
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
  print_thread_msg "BENCHMARKING ON ${omp_thread_count[$i]} CORES" "$paste_file"
  export OMP_NUM_THREADS="${omp_thread_count[$i]}"

  # 2.3.1. c-kzg [original]

  cd c-kzg/src || exit
  print_msg "c-kzg [original]" ../../"$paste_file"
  git restore Makefile
  git checkout main
  bench_c_kzg

  # 2.3.2. c-kzg [parallelized]

  print_msg "c-kzg [parallelized]" ../../"$paste_file"
  git checkout openmp
  eval "$(sed -i "s/KZG_CFLAGS =/KZG_CFLAGS = -fPIE -fopenmp/" Makefile)"
  eval "$(sed -i 's/KZG_CFLAGS += -O/KZG_CFLAGS += -Ofast/' Makefile)"
  bench_c_kzg
  cd ../..

  # rust crates
  cd kzg || exit

  # 2.3.3. arkworks

  print_msg "arkworks [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml >> ../"$paste_file"
  print_msg "arkworks [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path Arkworks/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.4. zkcrypto

  print_msg "zkcrypto [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml >> ../"$paste_file"
  print_msg "zkcrypto [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path zkcrypto/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.5. blst-from-scratch

  print_msg "blst-from-scratch [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml >> ../"$paste_file"
  print_msg "blst-from-scratch [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path blst-from-scratch/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.6. ckzg

  print_msg "ckzg [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path ckzg/Cargo.toml >> ../"$paste_file"
  print_msg "ckzg [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path ckzg/Cargo.toml --features parallel >> ../"$paste_file"

  # 2.3.7. mcl-kzg10-rust

  print_msg "mcl-kzg10-rust [original]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml >> ../"$paste_file"
  print_msg "mcl-kzg10-rust [parallelized]" ../"$paste_file"
  taskset --cpu-list "${taskset_cpu_list[$i]}" cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml --features mcl_rust/parallel >> ../"$paste_file"

  cd ..
  unset OMP_NUM_THREADS
done

# 3. collect results

paste_url=$(curl --upload-file "$paste_file" "https://paste.c-net.org/")

echo "=========================================="
echo "Uploaded to $paste_url"
echo "=========================================="
echo "Script finished in $(printf '%02dh:%02dm:%02ds\n' $((SECONDS/3600)) $((SECONDS%3600/60)) $((SECONDS%60)))"
echo "Success!"
