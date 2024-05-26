import re
import pandas as pd

normalize_benchmark_name = {
    'blob_to_kzg_commitment': 'blob_to_kzg_commitment',
    'Benchmark/BlobToKZGCommitment': 'blob_to_kzg_commitment',

    'compute_kzg_proof': 'compute_kzg_proof',
    'Benchmark/ComputeKZGProof': 'compute_kzg_proof',

    'compute_blob_kzg_proof': 'compute_blob_kzg_proof',
    'Benchmark/ComputeBlobKZGProof': 'compute_blob_kzg_proof',

    'verify_kzg_proof': 'verify_kzg_proof',
    'Benchmark/VerifyKZGProof': 'verify_kzg_proof',

    'verify_blob_kzg_proof': 'verify_blob_kzg_proof',
    'Benchmark/VerifyBlobKZGProof': 'verify_blob_kzg_proof',

    'verify_blob_kzg_proof_batch/1': 'verify_blob_kzg_proof_batch/1',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)': 'verify_blob_kzg_proof_batch/1',

    'verify_blob_kzg_proof_batch/2': 'verify_blob_kzg_proof_batch/2',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)': 'verify_blob_kzg_proof_batch/2',

    'verify_blob_kzg_proof_batch/4': 'verify_blob_kzg_proof_batch/4',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)': 'verify_blob_kzg_proof_batch/4',

    'verify_blob_kzg_proof_batch/8': 'verify_blob_kzg_proof_batch/8',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)': 'verify_blob_kzg_proof_batch/8',

    'verify_blob_kzg_proof_batch/16': 'verify_blob_kzg_proof_batch/16',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)': 'verify_blob_kzg_proof_batch/16',

    'verify_blob_kzg_proof_batch/32': 'verify_blob_kzg_proof_batch/32',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)': 'verify_blob_kzg_proof_batch/32',
    
    'verify_blob_kzg_proof_batch/64': 'verify_blob_kzg_proof_batch/64',
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)': 'verify_blob_kzg_proof_batch/64',
    
    'Benchmark/VerifyBlobKZGProofBatch(count=1)': 'verify_blob_kzg_proof_batch/1 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=2)': 'verify_blob_kzg_proof_batch/2 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=4)': 'verify_blob_kzg_proof_batch/4 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=8)': 'verify_blob_kzg_proof_batch/8 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=16)': 'verify_blob_kzg_proof_batch/16 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=32)': 'verify_blob_kzg_proof_batch/32 (sequential)',
    'Benchmark/VerifyBlobKZGProofBatch(count=64)': 'verify_blob_kzg_proof_batch/64 (sequential)',
    'BenchmarkComputeChallenge': 'BenchmarkComputeChallenge',
    'BenchmarkDeserializeBlob': 'BenchmarkDeserializeBlob',

    "bench_DAS_extension scale: '15'": "bench_DAS_extension scale: '15'",
    "bench_fft_fr scale: '15'": "bench_fft_fr scale: '15'",
    "bench_fft_g1 scale: '15'": "bench_fft_g1 scale: '15'",
    "bench_fk_single_da scale: '14'": "bench_fk_single_da scale: '14'",
    "bench_fk_multi_da scale: '14'": "bench_fk_multi_da scale: '14'",
    "bench_commit_to_poly scale: '15'": "bench_commit_to_poly scale: '15'",
    "bench_compute_proof_single scale: '15'": "bench_compute_proof_single scale: '15'",
    "bench_g1_lincomb points: '4096'": "bench_g1_lincomb points: '4096'",
    "bench_new_poly_div scale: '15'": "bench_new_poly_div scale: '15'",
    "bench_recover scale: '15'": "bench_recover scale: '15'",
    "bench_zero_poly scale: '15'": "bench_zero_poly scale: '15'",
    "bench_g1_lincomb with precomputation points: '4096'": "bench_g1_lincomb with precomputation points: '4096'",
}

def parse_benchmark_group(file, name):
    data = ""
    while True:
        pos = file.tell()
        line = file.readline()
        if len(line) == 0:
            file.seek(pos)
            break;
        res = re.match(r"^~+ ([^~]+) ~+$", line)
        if not (res is None):
            file.seek(pos)
            break
        res = re.match(r"^\*+ BENCHMARKING ON (\d+) CORES \*+$", line)
        if not (res is None):
            file.seek(pos)
            break
        data += line
    data = data.strip()

    go_matches = re.findall(r"^(.+)\t *\d+\t *(\d+) ns\/op(?:\t *\d+ B\/op\t *\d+ allocs\/op)?$", data, flags=re.M)
    rust_matches = re.findall(r"^(\S.+)\s+time:\s*\[\d+.\d*\sm?s\s(\d+.\d*\sm?s)\s\d+.\d*\sm?s\]", data, flags=re.M)
    
    output = {}

    if len(go_matches) > 0:
        for (name, time) in go_matches:
            res = re.match(r"^([^-\s]+)(?:-\d+)?\s*$", name)
            name = res.group(1)

            normalized_name = normalize_benchmark_name.get(name)
            if normalized_name is None:
                print("Warning! Unrecognized benchmark name - ", name)
                normalized_name = name

            output[normalized_name] = int(time)
    elif len(rust_matches) > 0:
        for (name, time) in rust_matches:
            name = name.strip()
            time = re.match(r"^(\d+\.\d+) (m?s)$", time)

            if time.group(2) == 'ms':
                scale = 1000000
            elif time.group(2) == 's':
                scale = 1000000000
            else:
                print("Unrecognized time unit: ", time.group(2))
                continue
            
            normalized_name = normalize_benchmark_name.get(name)
            if normalized_name is None:
                print("Warning! Unrecognized benchmark name - ", name)
                normalized_name = name

            output[normalized_name] = int(float(time.group(1)) * scale)
    else:
        print("Unrecognized! ", name)

    return output


def parse_thread_group(file, threads):
    group = {}

    while True:
        pos = file.tell()
        line = file.readline()
        if len(line) == 0:
            file.seek(pos)
            break;
        res = re.match(r"^~+([^~]+)~+$", line)
        if res is None and len(line.strip()) != 0:
            file.seek(pos)
            break
        if not (res is None):
            group_name = res.group(1).strip()
            group[group_name] = parse_benchmark_group(file, group_name)
    
    return group

def main():
    with open("./input/rust-kzg-benchmarks.txt", "r") as bench_results, pd.ExcelWriter("./output/results.ods", mode="w", engine="odf") as output_writer:
        line = bench_results.readline()
        while len(line) > 0:
            result = re.match(r"^\*+ BENCHMARKING ON (\d+) CORES \*+$", line)
            if not (result is None):
                print("bench", result.group(1))
                group = parse_thread_group(bench_results, result.group(1))
                df = pd.DataFrame(data=group)
                df.to_excel(output_writer, sheet_name=f"{result.group(1)} cores")

            line = bench_results.readline()


if __name__ == '__main__':
    main()
