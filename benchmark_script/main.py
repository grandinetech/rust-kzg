import re
import pandas as pd
import subprocess
import os

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
    
    "bench_g1_lincomb points: '16384'": "bench_g1_lincomb points: '16384'",
    "bench_g1_lincomb with precomputation points: '16384'": "bench_g1_lincomb with precomputation points: '16384'",

    "bench_g1_lincomb points: '65536'": "bench_g1_lincomb points: '65536'",
    "bench_g1_lincomb with precomputation points: '65536'": "bench_g1_lincomb with precomputation points: '65536'",

    "bench_g1_lincomb points: '262144'": "bench_g1_lincomb points: '262144'",
    "bench_g1_lincomb with precomputation points: '262144'": "bench_g1_lincomb with precomputation points: '262144'",

    "bench_g1_lincomb points: '1048576'": "bench_g1_lincomb points: '1048576'",
    "bench_g1_lincomb with precomputation points: '1048576'": "bench_g1_lincomb with precomputation points: '1048576'",
}

benchmark_name_to_sheet_name = {
    'blob_to_kzg_commitment': 'Blob to KZG commitment',
    'compute_kzg_proof': 'Compute KZG proof',
    'compute_blob_kzg_proof': 'Compute blob KZG proof',
    'verify_kzg_proof': 'Verify KZG proof',
    'verify_blob_kzg_proof': 'Verify blob KZG proof',
    'verify_blob_kzg_proof_batch/1': 'Verify blob KZG proof batch (count=1)',
    'verify_blob_kzg_proof_batch/2': 'Verify blob KZG proof batch (count=2)',
    'verify_blob_kzg_proof_batch/4': 'Verify blob KZG proof batch (count=4)',
    'verify_blob_kzg_proof_batch/8': 'Verify blob KZG proof batch (count=8)',
    'verify_blob_kzg_proof_batch/16': 'Verify blob KZG proof batch (count=16)',
    'verify_blob_kzg_proof_batch/32': 'Verify blob KZG proof batch (count=32)',
    'verify_blob_kzg_proof_batch/64': 'Verify blob KZG proof batch (count=64)',
    "bench_DAS_extension scale: '15'": 'DAS extension (scale=15)',
    "bench_fft_fr scale: '15'": 'FFT Fr (scale=15)',
    "bench_fft_g1 scale: '15'": 'FFT G1 (scale=15)',
    "bench_fk_single_da scale: '14'": 'FK single (scale=14)',
    "bench_fk_multi_da scale: '14'": 'FK multi (scale=14)',
    "bench_commit_to_poly scale: '15'": 'Commit to poly (scale=15)',
    "bench_compute_proof_single scale: '15'": 'Compute proof single (scale=15)',
    "bench_g1_lincomb with precomputation points: '4096'": 'Multi-scalar multiplication (points=4096)',
    "bench_new_poly_div scale: '15'": 'New poly div (scale=15)',
    "bench_recover scale: '15'": 'Recover (scale=15)',
    "bench_zero_poly scale: '15'": 'Zero poly (scale=15)',
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
        res = re.match(r"^\*+ BENCHMARKING ([^\*]+) \*+$", line)
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

def replace_path(match, resolve_var):
    path = match.group(1)

    result = resolve_var(path.split(">"))

    return str(result)
    

def expand_template(in_filename, out_filename, resolve_var):
    with open(in_filename, "r") as in_file, open(out_filename, "w") as out_file:
        for line in in_file:
            out_line = re.sub(r'\$\{([^}]+)\}', lambda match: replace_path(match, resolve_var), line)
            out_file.write(out_line)

def generate_from_template(template_name, out_filename, resolve_var):
    expand_template(template_name, f"./output/{out_filename}.tex", resolve_var)
    status = subprocess.call(f"wsl xelatex -halt-on-error -output-directory=./output ./output/{out_filename}.tex", shell=True)
    if status != 0:
        print("Failed to generate graph pdf")
        return
    status = subprocess.call(f"wsl convert -verbose -density 300 -trim ./output/{out_filename}.pdf -quality 100 -flatten ./output/{out_filename}.jpg")
    if status != 0:
        print("Failed to convert graph pdf to image")
        return

def generate_eip_graph(out_filename, data, criteria, time_unit):
    max_time = None
    max_time_2 = None
    for cores in data:
        for backend in ['rust-kzg with blst backend (parallel, bgmw)', 'rust-kzg with zkcrypto backend (parallel)']:
            result = None
            for i in criteria:
                if i in data[cores][backend]:
                    result = data[cores][backend][i]
                    break;
            if max_time is None and not (result is None):
                max_time = result
            if not (max_time is None) and not (result is None) and max_time < result:
                max_time = result
        for backend in ['rust-kzg with blst backend (parallel, bgmw)', 'go-kzg-4844', 'rust binding (c-kzg-4844)', 'rust binding (rust-kzg with blst backend)', 'go binding (rust-kzg with blst backend)']:
            result = None
            for i in criteria:
                if i in data[cores][backend]:
                    result = data[cores][backend][i]
                    break;
            if max_time_2 is None and not (result is None):
                max_time_2 = result
            if not (max_time_2 is None) and not (result is None) and max_time_2 < result:
                max_time_2 = result

    time_scale = None
    if time_unit == 'ms':
        time_scale = 1 / 1000000
    elif time_unit == 's':
        time_scale = 1 / 1000000000

    max_time *= time_scale
    max_time *= 1.1
    max_time_2 *= time_scale
    max_time_2 *= 1.1

    def resolve_var(path):
        if len(path) == 1:
            if path[0] == 'max_time_2':
                return max_time_2
            if path[0] == 'max_time':
                return max_time
            if path[0] == 'time_unit':
                return time_unit

        obj = data
        for segment in path:
            obj = obj[segment]

        result = None
        for i in criteria:
            if i in obj:
                result = obj[i]
                break
        return result * time_scale

    generate_from_template("./input/graphs/eip_graph_template.tex", out_filename, resolve_var)

def generate_fft_graph(out_filename, data):
    max_time = None
    max_time_2 = None
    for cores in data:
        for backend in ['rust-kzg with blst backend (parallel, bgmw)', 'rust-kzg with arkworks backend (parallel, bgmw)', 'rust-kzg with constantine backend (parallel, bgmw)', 'rust-kzg with zkcrypto backend (parallel)']:
            result = data[cores][backend]["bench_fft_fr scale: '15'"]
            if max_time is None:
                max_time = result
            elif max_time < result:
                max_time = result
        for backend in ['rust-kzg with blst backend (parallel, bgmw)', 'rust-kzg with arkworks backend (parallel, bgmw)', 'rust-kzg with constantine backend (parallel, bgmw)', 'rust-kzg with zkcrypto backend (parallel)']:
            result = data[cores][backend]["bench_fft_g1 scale: '15'"]
            if max_time_2 is None:
                max_time_2 = result
            elif max_time_2 < result:
                max_time_2 = result
    time_unit = 'ms'
    time_scale = 1 / 1000000
    time_unit_2 = 's'
    time_scale_2 = 1 / 1000000000

    max_time *= time_scale
    max_time *= 1.1
    max_time_2 *= time_scale_2
    max_time_2 *= 1.1

    def resolve_var(path):
        if len(path) == 1:
            if path[0] == 'max_time_2':
                return max_time_2
            if path[0] == 'max_time':
                return max_time
            if path[0] == 'time_unit':
                return time_unit
            if path[0] == 'time_unit_2':
                return time_unit_2

        result = data
        for segment in path:
            result = result[segment]

        if path[len(path) - 1] == "bench_fft_g1 scale: '15'":
            return result * time_scale_2

        return result * time_scale

    generate_from_template("./input/graphs/fft_graph_template.tex", out_filename, resolve_var)

def generate_msm_graph(out_filename, data):
    max_time = None
    max_time_2 = None
    for cores in data:
        for backend in ['rust-kzg with blst backend (parallel)', 'rust-kzg with arkworks backend (parallel)', 'rust-kzg with constantine backend (parallel)', 'rust-kzg with zkcrypto backend (parallel)']:
            result = data[cores][backend]["bench_g1_lincomb points: '4096'"]
            if max_time is None:
                max_time = result
            elif max_time < result:
                max_time = result
        for backend in ['rust-kzg with blst backend (sequential)', 'rust-kzg with arkworks backend (sequential)', 'rust-kzg with constantine backend (sequential)', 'rust-kzg with zkcrypto backend (sequential)']:
            result = data[cores][backend]["bench_g1_lincomb points: '4096'"]
            if max_time_2 is None:
                max_time_2 = result
            elif max_time_2 < result:
                max_time_2 = result
    time_unit = 'ms'
    time_scale = 1 / 1000000

    max_time *= time_scale
    max_time *= 1.1
    max_time_2 *= time_scale
    max_time_2 *= 1.1

    def resolve_var(path):
        if len(path) == 1:
            if path[0] == 'max_time_2':
                return max_time_2
            if path[0] == 'max_time':
                return max_time
            if path[0] == 'time_unit':
                return time_unit

        obj = data
        for segment in path:
            obj = obj[segment]

        result = None
        for i in ["bench_g1_lincomb with precomputation points: '4096'", "bench_g1_lincomb points: '4096'"]:
            if i in obj:
                result = obj[i]
                break
        return result * time_scale

    generate_from_template("./input/graphs/msm_graph_template.tex", out_filename, resolve_var)

def parse_cuda_benches(input):
    with open(f"./input/{input}") as bench_results:
        line = bench_results.readline()

        groups = {}
        name = ''

        while len(line) > 0:
            result = re.match(r"^~+([^~]+)~+$", line)

            if not (result is None):
                group_name = result.group(1).strip()

                group = parse_benchmark_group(bench_results, group_name)

                if group_name in groups:
                    groups[group_name] = groups[group_name] | group
                else:
                    groups[group_name] = group

            result = re.match(r"^\|\s+0\s+(.+)Off \|.+\|.+\|$", line)
            if not (result is None):
                name = result.group(1).strip()

            line = bench_results.readline()

        return (name, groups)

def generate_cuda_eip_graph(out_filename, data, criteria, time_unit):
    time_scale = None
    if time_unit == 'ms':
        time_scale = 1 / 1000000
    elif time_unit == 's':
        time_scale = 1 / 1000000000

    def resolve_var(path):
        if len(path) == 1:
            if path[0] == 'time_unit':
                return time_unit

        obj = data
        for segment in path:
            obj = obj[segment]

        result = None
        for i in criteria:
            if i in obj:
                result = obj[i]
                break

        if result is None:
            return '0'
        return result * time_scale

    generate_from_template("./input/graphs/cuda_eip_graph_template.tex", out_filename, resolve_var)

def generate_cuda_msm_graph(data, time_unit):
    time_scale = None
    if time_unit == 'ms':
        time_scale = 1 / 1000000
    elif time_unit == 's':
        time_scale = 1 / 1000000000

    def resolve_var(path):
        if len(path) == 1:
            if path[0] == 'time_unit':
                return time_unit

        obj = data
        for segment in path:
            obj = obj[segment]

        if obj is None:
            return '0'
        return obj * time_scale

    generate_from_template("./input/graphs/cuda_msm_graph_template.tex", "cuda_msm", resolve_var)

def main():
    if not os.path.exists("./output"):
        os.makedirs("./output")
    
    with open("./input/rust-kzg-benchmarks.txt", "r") as bench_results, pd.ExcelWriter("./output/results.ods", mode="w", engine="odf") as output_writer:
        groups = {}
        
        line = bench_results.readline()
        while len(line) > 0:
            result = re.match(r"^\*+ BENCHMARKING ON (\d+) CORES \*+$", line)
            if not (result is None):
                group = parse_thread_group(bench_results, result.group(1))
                groups[result.group(1)] = group
                df = pd.DataFrame(data=group)
                df.to_excel(output_writer, sheet_name=f"{result.group(1)} cores")

            line = bench_results.readline()

        # additional aggregation sheets
        aggr_groups = [
            ['blob_to_kzg_commitment'],
            ['compute_kzg_proof'],
            ['compute_blob_kzg_proof'],
            ['verify_kzg_proof'],
            ['verify_blob_kzg_proof'],
            ['verify_blob_kzg_proof_batch/1', 'verify_blob_kzg_proof_batch/1 (sequential)'], # fallback to sequential version
            ['verify_blob_kzg_proof_batch/2', 'verify_blob_kzg_proof_batch/2 (sequential)'],
            ['verify_blob_kzg_proof_batch/4', 'verify_blob_kzg_proof_batch/4 (sequential)'],
            ['verify_blob_kzg_proof_batch/8', 'verify_blob_kzg_proof_batch/8 (sequential)'],
            ['verify_blob_kzg_proof_batch/16', 'verify_blob_kzg_proof_batch/16 (sequential)'],
            ['verify_blob_kzg_proof_batch/32', 'verify_blob_kzg_proof_batch/32 (sequential)'],
            ['verify_blob_kzg_proof_batch/64', 'verify_blob_kzg_proof_batch/64 (sequential)'],
            ["bench_DAS_extension scale: '15'"],
            ["bench_fft_fr scale: '15'"],
            ["bench_fft_g1 scale: '15'"],
            ["bench_fk_single_da scale: '14'"],
            ["bench_fk_multi_da scale: '14'"],
            ["bench_commit_to_poly scale: '15'"],
            ["bench_compute_proof_single scale: '15'"],
            ["bench_g1_lincomb with precomputation points: '4096'", "bench_g1_lincomb points: '4096'"], # fallback to lincomb without precomputation
            ["bench_new_poly_div scale: '15'"],
            ["bench_recover scale: '15'"],
            ["bench_zero_poly scale: '15'"],
        ]

        aggregated_data = {}

        for aggrs in aggr_groups:
            aggregated_data[aggrs[0]] = {}
            for group in groups:
                for backend in groups[group]:
                    result = None
                    for aggr in aggrs:
                        if aggr in groups[group][backend]:
                            result = groups[group][backend][aggr]
                            break
                    if result is None:
                        print("Skipping \"" + aggr + "\" for " + backend + " backend")
                        continue
                    if not (backend in aggregated_data[aggrs[0]]):
                        aggregated_data[aggrs[0]][backend] = {}
                    aggregated_data[aggrs[0]][backend][group] = result
        for aggregate in aggregated_data:
            df = pd.DataFrame(data=aggregated_data[aggregate])
            df.to_excel(output_writer, sheet_name=benchmark_name_to_sheet_name[aggregate])

        eip_graphs = [
            ('blob_to_kzg_commitment', ['blob_to_kzg_commitment']), 
            ('blob_to_kzg_commitment', ['blob_to_kzg_commitment']),
            ('compute_kzg_proof', ['compute_kzg_proof']),
            ('compute_blob_kzg_proof', ['compute_blob_kzg_proof']),
            ('verify_kzg_proof', ['verify_kzg_proof']),
            ('verify_blob_kzg_proof', ['verify_blob_kzg_proof']),
            ('verify_blob_kzg_proof_batch_1', ['verify_blob_kzg_proof_batch/1', 'verify_blob_kzg_proof_batch/1 (sequential)']), # fallback to sequential version
            ('verify_blob_kzg_proof_batch_2', ['verify_blob_kzg_proof_batch/2', 'verify_blob_kzg_proof_batch/2 (sequential)']),
            ('verify_blob_kzg_proof_batch_4', ['verify_blob_kzg_proof_batch/4', 'verify_blob_kzg_proof_batch/4 (sequential)']),
            ('verify_blob_kzg_proof_batch_8', ['verify_blob_kzg_proof_batch/8', 'verify_blob_kzg_proof_batch/8 (sequential)']),
            ('verify_blob_kzg_proof_batch_16', ['verify_blob_kzg_proof_batch/16', 'verify_blob_kzg_proof_batch/16 (sequential)']),
            ('verify_blob_kzg_proof_batch_32', ['verify_blob_kzg_proof_batch/32', 'verify_blob_kzg_proof_batch/32 (sequential)']),
            ('verify_blob_kzg_proof_batch_64', ['verify_blob_kzg_proof_batch/64', 'verify_blob_kzg_proof_batch/64 (sequential)']),
        ]

        for (graph_name, criteria) in eip_graphs:
            generate_eip_graph(graph_name, groups, criteria, 'ms')
        generate_fft_graph('fft', groups)
        generate_msm_graph('multi_scalar_multiplication', groups)

        (name, benches) = parse_cuda_benches('rust-kzg-benchmarks-T4.txt')
        df = pd.DataFrame(data=benches)
        df.to_excel(output_writer, sheet_name=f"GPU {name}")
        groups[name] = benches

        (name, benches) = parse_cuda_benches('rust-kzg-benchmarks-L4.txt')
        df = pd.DataFrame(data=benches)
        df.to_excel(output_writer, sheet_name=f"GPU {name}")
        groups[name] = benches

        for (graph_name, criteria) in eip_graphs:
            generate_cuda_eip_graph('cuda_' + graph_name, groups, criteria, 'ms')
        generate_cuda_msm_graph(groups, 's')

if __name__ == '__main__':
    main()
