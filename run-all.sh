#!/bin/bash

paste_name="linode_debian_eu_central_cpu_$(nproc)"
paste_file="$paste_name.txt"

# 1.2. post shell restart

rustup install 1.51
rustup default 1.51

neofetch >> "$paste_file"

# 2. run individual benches

# 2.1. c-kzg [prep-up]

git clone https://github.com/supranational/blst.git
cd blst/
export CFLAGS="-Ofast -fno-builtin-memcpy -fPIC -Wall -Wextra -Werror"
bash build.sh
unset CFLAGS
cd ..
git clone https://github.com/tesa4436/c-kzg.git
cp -r blst/* c-kzg/lib/
cp -r blst/bindings/*.h c-kzg/inc/
cd c-kzg/src/

# 2.1.1. c-kzg [original]

printf "\n\n\n~~~~~~~~~~ c-kzg [original]~~~~~~~~~~\n\n\n" >> ../../"$paste_file"
make fft_fr_bench fft_g1_bench recover_bench zero_poly_bench kzg_proofs_bench poly_bench das_extension_bench >> ../../"$paste_file"

# 2.1.2. c-kzg [parallelized]

printf "\n\n\n~~~~~~~~~~ c-kzg [parallelized]~~~~~~~~~~\n\n\n" >> ../../"$paste_file"
git checkout openmp
eval "$(sed -i "s/KZG_CFLAGS =/KZG_CFLAGS = -fPIE -fopenmp/" Makefile)"
eval "$(sed -i 's/KZG_CFLAGS += -O/KZG_CFLAGS += -Ofast/' Makefile)"
OMP_NUM_THREADS=$(nproc) make fft_fr_bench fft_g1_bench recover_bench zero_poly_bench kzg_proofs_bench poly_bench das_extension_bench >> ../../"$paste_file"
cd ../../

# 2.2. kzg [prep-up]

git clone --single-branch --branch research https://github.com/sifraitech/kzg
cd kzg/

# 2.2.1. arkworks

printf "\n\n\n~~~~~~~~~~ arkworks [original]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path Arkworks/Cargo.toml >> ../"$paste_file"
printf "\n\n\n~~~~~~~~~~ arkworks [parallelized]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path Arkworks/Cargo.toml --features parallel >> ../"$paste_file"

# 2.2.2. zkcrypto

printf "\n\n\n~~~~~~~~~~ zkcrypto [original]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path zkcrypto/Cargo.toml >> ../"$paste_file"
printf "\n\n\n~~~~~~~~~~ zkcrypto [parallelized]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path zkcrypto/Cargo.toml --features parallel >> ../"$paste_file"

# 2.2.3. blst-from-scratch

printf "\n\n\n~~~~~~~~~~ blst-from-scratch [original]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path blst-from-scratch/Cargo.toml >> ../"$paste_file"
printf "\n\n\n~~~~~~~~~~ blst-from-scratch [parallelized]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path blst-from-scratch/Cargo.toml --features parallel >> ../"$paste_file"

# 2.2.4. ckzg

export "LIBOMP_PATH=$(find /usr/lib/llvm* -name libiomp5.so | head -n 1)"
export "RUSTFLAGS=-C link-arg=""$LIBOMP_PATH"""
export "OMP_NUM_THREADS=$(nproc)"
cd ckzg && bash build.sh
cd ..

printf "\n\n\n~~~~~~~~~~ ckzg [original]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path ckzg/Cargo.toml >> ../"$paste_file"
printf "\n\n\n~~~~~~~~~~ ckzg [parallelized]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path ckzg/Cargo.toml --features parallel >> ../"$paste_file"

# 2.2.5. mcl-kzg10-rust

cd mcl-kzg/kzg && bash build.sh
cd ../..

printf "\n\n\n~~~~~~~~~~ mcl-kzg10-rust [original]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml >> ../"$paste_file"
printf "\n\n\n~~~~~~~~~~ mcl-kzg10-rust [parallelized]~~~~~~~~~~\n\n\n" >> ../"$paste_file"
cargo bench --manifest-path mcl-kzg/kzg-bench/Cargo.toml --features mcl_rust/parallel >> ../"$paste_file"
cd ..

# 3. collect results

curl -d "api_paste_code=$(cat "$paste_file")" \
     -d "api_paste_private=1" \
     -d "api_paste_name=$paste_name" \
     -d "api_dev_key=ba2fd41ca7923844193bf05d3b19ed32" \
     -d "api_option=paste" "https://pastebin.com/api/api_post.php"
