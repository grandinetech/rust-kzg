"""""
Preprocessing of the benchmark results file is necessary:
* rust: fit rust benchmarks time into a single line separated by a whitespace
"""""

INPUT_DIR = 'input/'
OUTPUT_DIR = 'output/'

BENCH_FILE = 'TendencyDoable-edited'

RUST_KZG_GRAPH_TEMPLATE_FILE = 'graphs/rust_kzg_graph_template.tex'
OTHER_GRAPH_TEMPLATE_FILE = 'graphs/other_graph_template.tex'
OTHER_EXTRA_GRAPH_TEMPLATE_FILE = 'graphs/other_extra_graph_template.tex'

RUST_KZG_TABLE_TEMPLATE_FILE = 'tables/rust_kzg_table_template.tex'
OTHER_TABLE_TEMPLATE_FILE = 'tables/other_table_template.tex'
OTHER_EXTRA_TABLE_TEMPLATE_FILE = 'tables/other_extra_table_template.tex'

RUST_KZG_BENCH_COUNT = 21  # -2 fk20 benchmarks
GO_KZG_BENCH_COUNT = 19
OTHER_BENCH_COUNT = 12

# ----------------- cpu 1 -----------------
c_kzg_4844_rust_binding_cpu_1 = []
go_kzg_4844_cpu_1 = []
arkworks_original_cpu_1 = []
arkworks_parallelized_cpu_1 = []
zkcrypto_original_cpu_1 = []
zkcrypto_parallelized_cpu_1 = []
blst_from_scratch_original_cpu_1 = []
blst_from_scratch_parallelized_cpu_1 = []
mcl_original_cpu_1 = []
mcl_parallelized_cpu_1 = []
blst_from_scratch_rust_binding_cpu_1 = []
blst_from_scratch_go_binding_cpu_1 = []
# ----------------- cpu 2 -----------------
c_kzg_4844_rust_binding_cpu_2 = []
go_kzg_4844_cpu_2 = []
arkworks_original_cpu_2 = []
arkworks_parallelized_cpu_2 = []
zkcrypto_original_cpu_2 = []
zkcrypto_parallelized_cpu_2 = []
blst_from_scratch_original_cpu_2 = []
blst_from_scratch_parallelized_cpu_2 = []
mcl_original_cpu_2 = []
mcl_parallelized_cpu_2 = []
blst_from_scratch_rust_binding_cpu_2 = []
blst_from_scratch_go_binding_cpu_2 = []
# ----------------- cpu 4 -----------------
c_kzg_4844_rust_binding_cpu_4 = []
go_kzg_4844_cpu_4 = []
arkworks_original_cpu_4 = []
arkworks_parallelized_cpu_4 = []
zkcrypto_original_cpu_4 = []
zkcrypto_parallelized_cpu_4 = []
blst_from_scratch_original_cpu_4 = []
blst_from_scratch_parallelized_cpu_4 = []
mcl_original_cpu_4 = []
mcl_parallelized_cpu_4 = []
blst_from_scratch_rust_binding_cpu_4 = []
blst_from_scratch_go_binding_cpu_4 = []
# ----------------- cpu 8 -----------------
c_kzg_4844_rust_binding_cpu_8 = []
go_kzg_4844_cpu_8 = []
arkworks_original_cpu_8 = []
arkworks_parallelized_cpu_8 = []
zkcrypto_original_cpu_8 = []
zkcrypto_parallelized_cpu_8 = []
blst_from_scratch_original_cpu_8 = []
blst_from_scratch_parallelized_cpu_8 = []
mcl_original_cpu_8 = []
mcl_parallelized_cpu_8 = []
blst_from_scratch_rust_binding_cpu_8 = []
blst_from_scratch_go_binding_cpu_8 = []
# ----------------- cpu 16 -----------------
c_kzg_4844_rust_binding_cpu_16 = []
go_kzg_4844_cpu_16 = []
arkworks_original_cpu_16 = []
arkworks_parallelized_cpu_16 = []
zkcrypto_original_cpu_16 = []
zkcrypto_parallelized_cpu_16 = []
blst_from_scratch_original_cpu_16 = []
blst_from_scratch_parallelized_cpu_16 = []
mcl_original_cpu_16 = []
mcl_parallelized_cpu_16 = []
blst_from_scratch_rust_binding_cpu_16 = []
blst_from_scratch_go_binding_cpu_16 = []


def action_go(line: str):
    split_str = line.split(' ')
    time_value = float(split_str[-2]) / 1000000
    # --------------------------- cpu 1 ---------------------------
    if len(go_kzg_4844_cpu_1) < GO_KZG_BENCH_COUNT:
        go_kzg_4844_cpu_1.append(time_value)
    elif len(blst_from_scratch_go_binding_cpu_1) < OTHER_BENCH_COUNT:
        blst_from_scratch_go_binding_cpu_1.append(time_value)
    # --------------------------- cpu 2 ---------------------------
    elif len(go_kzg_4844_cpu_2) < GO_KZG_BENCH_COUNT:
        go_kzg_4844_cpu_2.append(time_value)
    elif len(blst_from_scratch_go_binding_cpu_2) < OTHER_BENCH_COUNT:
        blst_from_scratch_go_binding_cpu_2.append(time_value)
    # --------------------------- cpu 4 ---------------------------
    elif len(go_kzg_4844_cpu_4) < GO_KZG_BENCH_COUNT:
        go_kzg_4844_cpu_4.append(time_value)
    elif len(blst_from_scratch_go_binding_cpu_4) < OTHER_BENCH_COUNT:
        blst_from_scratch_go_binding_cpu_4.append(time_value)
    # --------------------------- cpu 8 ---------------------------
    elif len(go_kzg_4844_cpu_8) < GO_KZG_BENCH_COUNT:
        go_kzg_4844_cpu_8.append(time_value)
    elif len(blst_from_scratch_go_binding_cpu_8) < OTHER_BENCH_COUNT:
        blst_from_scratch_go_binding_cpu_8.append(time_value)
    # --------------------------- cpu 16 ---------------------------
    elif len(go_kzg_4844_cpu_16) < GO_KZG_BENCH_COUNT:
        go_kzg_4844_cpu_16.append(time_value)
    elif len(blst_from_scratch_go_binding_cpu_16) < OTHER_BENCH_COUNT:
        blst_from_scratch_go_binding_cpu_16.append(time_value)


def action_rust(line: str):
    split_str = line.split(' ')

    # take middle value
    time_value = float(split_str[-4])
    time_unit_str = split_str[-3]
    if time_unit_str == 's':
        time_value *= 1000

    # --------------------------- cpu 1 ---------------------------
    if len(c_kzg_4844_rust_binding_cpu_1) < OTHER_BENCH_COUNT:
        c_kzg_4844_rust_binding_cpu_1.append(time_value)
    elif len(arkworks_original_cpu_1) < RUST_KZG_BENCH_COUNT:
        arkworks_original_cpu_1.append(time_value)
    elif len(arkworks_parallelized_cpu_1) < RUST_KZG_BENCH_COUNT:
        arkworks_parallelized_cpu_1.append(time_value)
    elif len(zkcrypto_original_cpu_1) < RUST_KZG_BENCH_COUNT:
        zkcrypto_original_cpu_1.append(time_value)
    elif len(zkcrypto_parallelized_cpu_1) < RUST_KZG_BENCH_COUNT:
        zkcrypto_parallelized_cpu_1.append(time_value)
    elif len(blst_from_scratch_original_cpu_1) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_original_cpu_1.append(time_value)
    elif len(blst_from_scratch_parallelized_cpu_1) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_parallelized_cpu_1.append(time_value)
    elif len(mcl_original_cpu_1) < RUST_KZG_BENCH_COUNT:
        mcl_original_cpu_1.append(time_value)
    elif len(mcl_parallelized_cpu_1) < RUST_KZG_BENCH_COUNT:
        mcl_parallelized_cpu_1.append(time_value)
    elif len(blst_from_scratch_rust_binding_cpu_1) < OTHER_BENCH_COUNT:
        blst_from_scratch_rust_binding_cpu_1.append(time_value)
    # --------------------------- cpu 2 ---------------------------
    elif len(c_kzg_4844_rust_binding_cpu_2) < OTHER_BENCH_COUNT:
        c_kzg_4844_rust_binding_cpu_2.append(time_value)
    elif len(arkworks_original_cpu_2) < RUST_KZG_BENCH_COUNT:
        arkworks_original_cpu_2.append(time_value)
    elif len(arkworks_parallelized_cpu_2) < RUST_KZG_BENCH_COUNT:
        arkworks_parallelized_cpu_2.append(time_value)
    elif len(zkcrypto_original_cpu_2) < RUST_KZG_BENCH_COUNT:
        zkcrypto_original_cpu_2.append(time_value)
    elif len(zkcrypto_parallelized_cpu_2) < RUST_KZG_BENCH_COUNT:
        zkcrypto_parallelized_cpu_2.append(time_value)
    elif len(blst_from_scratch_original_cpu_2) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_original_cpu_2.append(time_value)
    elif len(blst_from_scratch_parallelized_cpu_2) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_parallelized_cpu_2.append(time_value)
    elif len(mcl_original_cpu_2) < RUST_KZG_BENCH_COUNT:
        mcl_original_cpu_2.append(time_value)
    elif len(mcl_parallelized_cpu_2) < RUST_KZG_BENCH_COUNT:
        mcl_parallelized_cpu_2.append(time_value)
    elif len(blst_from_scratch_rust_binding_cpu_2) < OTHER_BENCH_COUNT:
        blst_from_scratch_rust_binding_cpu_2.append(time_value)
    # --------------------------- cpu 4 ---------------------------
    elif len(c_kzg_4844_rust_binding_cpu_4) < OTHER_BENCH_COUNT:
        c_kzg_4844_rust_binding_cpu_4.append(time_value)
    elif len(arkworks_original_cpu_4) < RUST_KZG_BENCH_COUNT:
        arkworks_original_cpu_4.append(time_value)
    elif len(arkworks_parallelized_cpu_4) < RUST_KZG_BENCH_COUNT:
        arkworks_parallelized_cpu_4.append(time_value)
    elif len(zkcrypto_original_cpu_4) < RUST_KZG_BENCH_COUNT:
        zkcrypto_original_cpu_4.append(time_value)
    elif len(zkcrypto_parallelized_cpu_4) < RUST_KZG_BENCH_COUNT:
        zkcrypto_parallelized_cpu_4.append(time_value)
    elif len(blst_from_scratch_original_cpu_4) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_original_cpu_4.append(time_value)
    elif len(blst_from_scratch_parallelized_cpu_4) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_parallelized_cpu_4.append(time_value)
    elif len(mcl_original_cpu_4) < RUST_KZG_BENCH_COUNT:
        mcl_original_cpu_4.append(time_value)
    elif len(mcl_parallelized_cpu_4) < RUST_KZG_BENCH_COUNT:
        mcl_parallelized_cpu_4.append(time_value)
    elif len(blst_from_scratch_rust_binding_cpu_4) < OTHER_BENCH_COUNT:
        blst_from_scratch_rust_binding_cpu_4.append(time_value)
    # --------------------------- cpu 8 ---------------------------
    elif len(c_kzg_4844_rust_binding_cpu_8) < OTHER_BENCH_COUNT:
        c_kzg_4844_rust_binding_cpu_8.append(time_value)
    elif len(arkworks_original_cpu_8) < RUST_KZG_BENCH_COUNT:
        arkworks_original_cpu_8.append(time_value)
    elif len(arkworks_parallelized_cpu_8) < RUST_KZG_BENCH_COUNT:
        arkworks_parallelized_cpu_8.append(time_value)
    elif len(zkcrypto_original_cpu_8) < RUST_KZG_BENCH_COUNT:
        zkcrypto_original_cpu_8.append(time_value)
    elif len(zkcrypto_parallelized_cpu_8) < RUST_KZG_BENCH_COUNT:
        zkcrypto_parallelized_cpu_8.append(time_value)
    elif len(blst_from_scratch_original_cpu_8) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_original_cpu_8.append(time_value)
    elif len(blst_from_scratch_parallelized_cpu_8) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_parallelized_cpu_8.append(time_value)
    elif len(mcl_original_cpu_8) < RUST_KZG_BENCH_COUNT:
        mcl_original_cpu_8.append(time_value)
    elif len(mcl_parallelized_cpu_8) < RUST_KZG_BENCH_COUNT:
        mcl_parallelized_cpu_8.append(time_value)
    elif len(blst_from_scratch_rust_binding_cpu_8) < OTHER_BENCH_COUNT:
        blst_from_scratch_rust_binding_cpu_8.append(time_value)
    # --------------------------- cpu 16 ---------------------------
    elif len(c_kzg_4844_rust_binding_cpu_16) < OTHER_BENCH_COUNT:
        c_kzg_4844_rust_binding_cpu_16.append(time_value)
    elif len(arkworks_original_cpu_16) < RUST_KZG_BENCH_COUNT:
        arkworks_original_cpu_16.append(time_value)
    elif len(arkworks_parallelized_cpu_16) < RUST_KZG_BENCH_COUNT:
        arkworks_parallelized_cpu_16.append(time_value)
    elif len(zkcrypto_original_cpu_16) < RUST_KZG_BENCH_COUNT:
        zkcrypto_original_cpu_16.append(time_value)
    elif len(zkcrypto_parallelized_cpu_16) < RUST_KZG_BENCH_COUNT:
        zkcrypto_parallelized_cpu_16.append(time_value)
    elif len(blst_from_scratch_original_cpu_16) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_original_cpu_16.append(time_value)
    elif len(blst_from_scratch_parallelized_cpu_16) < RUST_KZG_BENCH_COUNT:
        blst_from_scratch_parallelized_cpu_16.append(time_value)
    elif len(mcl_original_cpu_16) < RUST_KZG_BENCH_COUNT:
        mcl_original_cpu_16.append(time_value)
    elif len(mcl_parallelized_cpu_16) < RUST_KZG_BENCH_COUNT:
        mcl_parallelized_cpu_16.append(time_value)
    elif len(blst_from_scratch_rust_binding_cpu_16) < OTHER_BENCH_COUNT:
        blst_from_scratch_rust_binding_cpu_16.append(time_value)


BENCH_STR = {
    # go, 1 core
    'Benchmark/BlobToKZGCommitment         ':                 action_go,
    'Benchmark/ComputeKZGProof             ':                 action_go,
    'Benchmark/ComputeBlobKZGProof         ':                 action_go,
    'Benchmark/VerifyKZGProof              ':                 action_go,
    'Benchmark/VerifyBlobKZGProof          ':                 action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=1)         ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=2)         ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=4)         ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=8)         ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=16)        ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=32)        ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=64)        ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)      ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)      ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)      ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)      ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)     ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)     ':    action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)     ':    action_go,
    # go, 2 core
    'Benchmark/BlobToKZGCommitment-2         ':               action_go,
    'Benchmark/ComputeKZGProof-2             ':               action_go,
    'Benchmark/ComputeBlobKZGProof-2         ':               action_go,
    'Benchmark/VerifyKZGProof-2              ':               action_go,
    'Benchmark/VerifyBlobKZGProof-2          ':               action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=1)-2         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=2)-2         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=4)-2         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=8)-2         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=16)-2        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=32)-2        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=64)-2        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)-2      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)-2      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)-2      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)-2      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)-2     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)-2     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)-2     ':  action_go,
    # go, 4 core
    'Benchmark/BlobToKZGCommitment-4         ':               action_go,
    'Benchmark/ComputeKZGProof-4             ':               action_go,
    'Benchmark/ComputeBlobKZGProof-4         ':               action_go,
    'Benchmark/VerifyKZGProof-4              ':               action_go,
    'Benchmark/VerifyBlobKZGProof-4          ':               action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=1)-4         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=2)-4         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=4)-4         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=8)-4         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=16)-4        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=32)-4        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=64)-4        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)-4      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)-4      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)-4      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)-4      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)-4     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)-4     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)-4     ':  action_go,
    # go, 8 core
    'Benchmark/BlobToKZGCommitment-8         ':               action_go,
    'Benchmark/ComputeKZGProof-8             ':               action_go,
    'Benchmark/ComputeBlobKZGProof-8         ':               action_go,
    'Benchmark/VerifyKZGProof-8              ':               action_go,
    'Benchmark/VerifyBlobKZGProof-8          ':               action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=1)-8         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=2)-8         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=4)-8         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=8)-8         ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=16)-8        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=32)-8        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=64)-8        ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)-8      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)-8      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)-8      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)-8      ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)-8     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)-8     ':  action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)-8     ':  action_go,
    # go, 16 core
    'Benchmark/BlobToKZGCommitment-16         ':              action_go,
    'Benchmark/ComputeKZGProof-16             ':              action_go,
    'Benchmark/ComputeBlobKZGProof-16         ':              action_go,
    'Benchmark/VerifyKZGProof-16              ':              action_go,
    'Benchmark/VerifyBlobKZGProof-16          ':              action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=1)-16         ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=2)-16         ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=4)-16         ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=8)-16         ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=16)-16        ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=32)-16        ': action_go,
    'Benchmark/VerifyBlobKZGProofBatch(count=64)-16        ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=1)-16      ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=2)-16      ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=4)-16      ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=8)-16      ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=16)-16     ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=32)-16     ': action_go,
    'Benchmark/VerifyBlobKZGProofBatchPar(count=64)-16     ': action_go,
    # rust
    "bench_DAS_extension scale: '15' time":        action_rust,
    "blob_to_kzg_commitment  time":                action_rust,
    "compute_kzg_proof       time":                action_rust,
    "verify_kzg_proof        time":                action_rust,
    "compute_blob_kzg_proof  time":                action_rust,
    "verify_blob_kzg_proof   time":                action_rust,
    "verify_blob_kzg_proof_batch/1 time":          action_rust,
    "verify_blob_kzg_proof_batch/2 time":          action_rust,
    "verify_blob_kzg_proof_batch/4 time":          action_rust,
    "verify_blob_kzg_proof_batch/8 time":          action_rust,
    "verify_blob_kzg_proof_batch/16 time":         action_rust,
    "verify_blob_kzg_proof_batch/32 time":         action_rust,
    "verify_blob_kzg_proof_batch/64 time":         action_rust,
    "bench_fft_fr scale: '15' time":               action_rust,
    "bench_fft_g1 scale: '15' time":               action_rust,
    "bench_commit_to_poly scale: '15' time":       action_rust,
    "bench_compute_proof_single scale: '15' time": action_rust,
    "bench_g1_lincomb points: '4096' time":        action_rust,
    "bench_new_poly_div scale: '15' time":         action_rust,
    "bench_recover scale: '15' time":              action_rust,
    "bench_zero_poly scale: '15' time":            action_rust,
}


def check_go_kzg_condition(bench_list: list):
    var_name = f'{bench_list}'.partition('=')[0]
    assert len(bench_list) == GO_KZG_BENCH_COUNT, \
        f"{var_name} result count less than {GO_KZG_BENCH_COUNT} expected, got: {len(bench_list)}"


def check_other_condition(bench_list: list):
    var_name = f'{bench_list}'.partition('=')[0]
    assert len(bench_list) == OTHER_BENCH_COUNT, \
        f"{var_name} result count less than {OTHER_BENCH_COUNT} expected, got: {len(bench_list)}"


def check_rust_kzg_condition(bench_list: list):
    var_name = f'{bench_list}'.partition('=')[0]
    assert len(bench_list) == RUST_KZG_BENCH_COUNT, \
        f"{var_name} result count less than {RUST_KZG_BENCH_COUNT} expected, got: {len(bench_list)}"


def read_bench_file(bench_file: str):
    with open(INPUT_DIR + bench_file, 'r') as file:
        for line in file:
            for search, action in BENCH_STR.items():
                if search in line:
                    action(line)


def gen_other_graph(title: str, category: str, time_unit: str, unit_multiplier: float, max_time_to_override_with: float, rust_kzg_index: int, rust_binding_index: int, go_binding_index: int, extra_index=None):
    if extra_index is not None:
        graph_file = OTHER_EXTRA_GRAPH_TEMPLATE_FILE
    else:
        graph_file = OTHER_GRAPH_TEMPLATE_FILE

    with open(INPUT_DIR + graph_file, 'r') as file:
        filedata = file.read()

    filedata = filedata.replace('TITLE', title.replace('_', ' ') + ' (parallel)')
    filedata = filedata.replace('TIME_UNIT', 'time (' + time_unit + ')')

    # rust binding (c-kzg-4844)
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_1_CORES', str(unit_multiplier * c_kzg_4844_rust_binding_cpu_1[rust_binding_index]))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_2_CORES', str(unit_multiplier * c_kzg_4844_rust_binding_cpu_2[rust_binding_index]))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_4_CORES', str(unit_multiplier * c_kzg_4844_rust_binding_cpu_4[rust_binding_index]))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_8_CORES', str(unit_multiplier * c_kzg_4844_rust_binding_cpu_8[rust_binding_index]))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_16_CORES', str(unit_multiplier * c_kzg_4844_rust_binding_cpu_16[rust_binding_index]))
    # blst-from-scratch
    filedata = filedata.replace('BLST_PARALLELIZED_1_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_1[rust_kzg_index]))
    filedata = filedata.replace('BLST_PARALLELIZED_2_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_2[rust_kzg_index]))
    filedata = filedata.replace('BLST_PARALLELIZED_4_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_4[rust_kzg_index]))
    filedata = filedata.replace('BLST_PARALLELIZED_8_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_8[rust_kzg_index]))
    filedata = filedata.replace('BLST_PARALLELIZED_16_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_16[rust_kzg_index]))
    # rust binding (blst-from-scratch)
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_1_CORES', str(unit_multiplier * blst_from_scratch_rust_binding_cpu_1[rust_binding_index]))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_2_CORES', str(unit_multiplier * blst_from_scratch_rust_binding_cpu_2[rust_binding_index]))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_4_CORES', str(unit_multiplier * blst_from_scratch_rust_binding_cpu_4[rust_binding_index]))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_8_CORES', str(unit_multiplier * blst_from_scratch_rust_binding_cpu_8[rust_binding_index]))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_16_CORES', str(unit_multiplier * blst_from_scratch_rust_binding_cpu_16[rust_binding_index]))
    # go-kzg-4844
    filedata = filedata.replace('GO_KZG_4844_1_CORES', str(unit_multiplier * go_kzg_4844_cpu_1[go_binding_index]))
    filedata = filedata.replace('GO_KZG_4844_2_CORES', str(unit_multiplier * go_kzg_4844_cpu_2[go_binding_index]))
    filedata = filedata.replace('GO_KZG_4844_4_CORES', str(unit_multiplier * go_kzg_4844_cpu_4[go_binding_index]))
    filedata = filedata.replace('GO_KZG_4844_8_CORES', str(unit_multiplier * go_kzg_4844_cpu_8[go_binding_index]))
    filedata = filedata.replace('GO_KZG_4844_16_CORES', str(unit_multiplier * go_kzg_4844_cpu_16[go_binding_index]))
    # go binding (blst-from-scratch)
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_1_CORES', str(unit_multiplier * blst_from_scratch_go_binding_cpu_1[go_binding_index]))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_2_CORES', str(unit_multiplier * blst_from_scratch_go_binding_cpu_2[go_binding_index]))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_4_CORES', str(unit_multiplier * blst_from_scratch_go_binding_cpu_4[go_binding_index]))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_8_CORES', str(unit_multiplier * blst_from_scratch_go_binding_cpu_8[go_binding_index]))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_16_CORES', str(unit_multiplier * blst_from_scratch_go_binding_cpu_16[go_binding_index]))
    if extra_index is not None:
        # go-kzg-4844 extra index
        filedata = filedata.replace('GO_KZG_4844_EXTRA_1_CORES', str(unit_multiplier * go_kzg_4844_cpu_1[extra_index]))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_2_CORES', str(unit_multiplier * go_kzg_4844_cpu_2[extra_index]))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_4_CORES', str(unit_multiplier * go_kzg_4844_cpu_4[extra_index]))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_8_CORES', str(unit_multiplier * go_kzg_4844_cpu_8[extra_index]))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_16_CORES', str(unit_multiplier * go_kzg_4844_cpu_16[extra_index]))

    if extra_index is not None:
        extra_item = go_kzg_4844_cpu_1[extra_index]
    else:
        extra_item = 0

    max_time_list = [
        c_kzg_4844_rust_binding_cpu_1[rust_binding_index],
        blst_from_scratch_parallelized_cpu_1[rust_kzg_index],
        blst_from_scratch_rust_binding_cpu_1[rust_binding_index],
        go_kzg_4844_cpu_1[go_binding_index],
        blst_from_scratch_go_binding_cpu_1[go_binding_index],
        extra_item
    ]

    if max_time_to_override_with == 0:
        max_time = max(max_time_list) * 1.1
        filedata = filedata.replace('MAX_TIME', str(unit_multiplier * max_time))
    else:
        filedata = filedata.replace('MAX_TIME', str(max_time_to_override_with))

    with open(OUTPUT_DIR
              + title
                      .replace('(', '_')
                      .replace(')', '_')
                      .replace('=', '_')
              + "_"
              + category
              + '_graph.tex',
              'w') as file:
        file.write(filedata)


def gen_rust_kzg_graph(title: str, category: str, time_unit: str, unit_multiplier: float, max_time_to_override_with: float, index: int):
    with open(INPUT_DIR + RUST_KZG_GRAPH_TEMPLATE_FILE, 'r') as file:
        filedata = file.read()

    filedata = filedata.replace('TITLE', title.replace('_', ' ') + ' (parallel)')
    filedata = filedata.replace('TIME_UNIT', 'time (' + time_unit + ')')
    # zkcrypto
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_1_CORES', str(unit_multiplier * zkcrypto_parallelized_cpu_1[index]))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_2_CORES', str(unit_multiplier * zkcrypto_parallelized_cpu_2[index]))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_4_CORES', str(unit_multiplier * zkcrypto_parallelized_cpu_4[index]))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_8_CORES', str(unit_multiplier * zkcrypto_parallelized_cpu_8[index]))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_16_CORES', str(unit_multiplier * zkcrypto_parallelized_cpu_16[index]))
    # blst-from-scratch
    filedata = filedata.replace('BLST_PARALLELIZED_1_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_1[index]))
    filedata = filedata.replace('BLST_PARALLELIZED_2_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_2[index]))
    filedata = filedata.replace('BLST_PARALLELIZED_4_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_4[index]))
    filedata = filedata.replace('BLST_PARALLELIZED_8_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_8[index]))
    filedata = filedata.replace('BLST_PARALLELIZED_16_CORES', str(unit_multiplier * blst_from_scratch_parallelized_cpu_16[index]))
    # mcl
    filedata = filedata.replace('MCL_PARALLELIZED_1_CORES', str(unit_multiplier * mcl_parallelized_cpu_1[index]))
    filedata = filedata.replace('MCL_PARALLELIZED_2_CORES', str(unit_multiplier * mcl_parallelized_cpu_2[index]))
    filedata = filedata.replace('MCL_PARALLELIZED_4_CORES', str(unit_multiplier * mcl_parallelized_cpu_4[index]))
    filedata = filedata.replace('MCL_PARALLELIZED_8_CORES', str(unit_multiplier * mcl_parallelized_cpu_8[index]))
    filedata = filedata.replace('MCL_PARALLELIZED_16_CORES', str(unit_multiplier * mcl_parallelized_cpu_16[index]))

    max_time_list = [
        zkcrypto_parallelized_cpu_1[index],
        blst_from_scratch_parallelized_cpu_1[index],
        mcl_parallelized_cpu_1[index]
    ]

    if max_time_to_override_with == 0:
        max_time = max(max_time_list) * 1.1
        filedata = filedata.replace('MAX_TIME', str(unit_multiplier * max_time))
    else:
        filedata = filedata.replace('MAX_TIME', str(max_time_to_override_with))

    with open(OUTPUT_DIR
              + title
                      .replace('(', '_')
                      .replace(')', '_')
                      .replace('=', '_')
              + "_"
              + category
              + '_graph.tex',
              'w') as file:
        file.write(filedata)


def calc_delta(val_1: float, val_2: float) -> str:
    change = round((((val_2 - val_1) / val_1) * 100), 3)
    change_str = str(change)
    if change > 0:
        change_str = '+' + change_str
    return change_str


def gen_other_table(title: str, category: str, time_unit: str, unit_multiplier: float, rust_kzg_index: int, rust_binding_index: int, go_binding_index: int, extra_index=None):
    if extra_index is not None:
        graph_file = OTHER_EXTRA_TABLE_TEMPLATE_FILE
    else:
        graph_file = OTHER_TABLE_TEMPLATE_FILE

    with open(INPUT_DIR + graph_file, 'r') as file:
        filedata = file.read()

    filedata = filedata.replace('TITLE', title.replace('_', ' ') + ' (parallel)')
    filedata = filedata.replace('TIME_UNIT', 'Time (' + time_unit + ')')

    # rust binding (c-kzg-4844)
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_1_CORES', str(round(unit_multiplier * c_kzg_4844_rust_binding_cpu_1[rust_binding_index], 3)))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_2_CORES', str(round(unit_multiplier * c_kzg_4844_rust_binding_cpu_2[rust_binding_index], 3)))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_4_CORES', str(round(unit_multiplier * c_kzg_4844_rust_binding_cpu_4[rust_binding_index], 3)))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_8_CORES', str(round(unit_multiplier * c_kzg_4844_rust_binding_cpu_8[rust_binding_index], 3)))
    filedata = filedata.replace('C_KZG_4844_RUST_BINDING_16_CORES', str(round(unit_multiplier * c_kzg_4844_rust_binding_cpu_16[rust_binding_index], 3)))
    # blst-from-scratch
    filedata = filedata.replace('BLST_PARALLELIZED_1_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_1[rust_kzg_index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_2_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_2[rust_kzg_index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_4_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_4[rust_kzg_index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_8_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_8[rust_kzg_index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_16_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_16[rust_kzg_index], 3)))
    # rust binding (blst-from-scratch)
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_1_CORES', str(round(unit_multiplier * blst_from_scratch_rust_binding_cpu_1[rust_binding_index], 3)))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_2_CORES', str(round(unit_multiplier * blst_from_scratch_rust_binding_cpu_2[rust_binding_index], 3)))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_4_CORES', str(round(unit_multiplier * blst_from_scratch_rust_binding_cpu_4[rust_binding_index], 3)))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_8_CORES', str(round(unit_multiplier * blst_from_scratch_rust_binding_cpu_8[rust_binding_index], 3)))
    filedata = filedata.replace('RUST_BINDING_PARALLELIZED_16_CORES', str(round(unit_multiplier * blst_from_scratch_rust_binding_cpu_16[rust_binding_index], 3)))
    # go-kzg-4844
    filedata = filedata.replace('GO_KZG_4844_1_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_1[go_binding_index], 3)))
    filedata = filedata.replace('GO_KZG_4844_2_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_2[go_binding_index], 3)))
    filedata = filedata.replace('GO_KZG_4844_4_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_4[go_binding_index], 3)))
    filedata = filedata.replace('GO_KZG_4844_8_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_8[go_binding_index], 3)))
    filedata = filedata.replace('GO_KZG_4844_16_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_16[go_binding_index], 3)))
    # go binding (blst-from-scratch)
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_1_CORES', str(round(unit_multiplier * blst_from_scratch_go_binding_cpu_1[go_binding_index], 3)))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_2_CORES', str(round(unit_multiplier * blst_from_scratch_go_binding_cpu_2[go_binding_index], 3)))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_4_CORES', str(round(unit_multiplier * blst_from_scratch_go_binding_cpu_4[go_binding_index], 3)))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_8_CORES', str(round(unit_multiplier * blst_from_scratch_go_binding_cpu_8[go_binding_index], 3)))
    filedata = filedata.replace('GO_BINDING_PARALLELIZED_16_CORES', str(round(unit_multiplier * blst_from_scratch_go_binding_cpu_16[go_binding_index], 3)))
    if extra_index is not None:
        # go-kzg-4844 extra index
        filedata = filedata.replace('GO_KZG_4844_EXTRA_1_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_1[extra_index], 3)))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_2_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_2[extra_index], 3)))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_4_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_4[extra_index], 3)))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_8_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_8[extra_index], 3)))
        filedata = filedata.replace('GO_KZG_4844_EXTRA_16_CORES', str(round(unit_multiplier * go_kzg_4844_cpu_16[extra_index], 3)))

    with open(OUTPUT_DIR
              + title
                      .replace('(', '_')
                      .replace(')', '_')
                      .replace('=', '_')
              + "_"
              + category
              + '_table.tex',
              'w') as file:
        file.write(filedata)


def gen_rust_kzg_table(title: str, category: str, time_unit: str, unit_multiplier: float, index: int):
    with open(INPUT_DIR + RUST_KZG_TABLE_TEMPLATE_FILE, 'r') as file:
        filedata = file.read()

    filedata = filedata.replace('TITLE', title.replace('_', ' ') + ' (parallel)')
    filedata = filedata.replace('TIME_UNIT', 'time (' + time_unit + ')')
    # zkcrypto
    filedata = filedata.replace('ZKCRYPTO_ORIGINAL_1_CORES', str(round(unit_multiplier * zkcrypto_original_cpu_1[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_ORIGINAL_2_CORES', str(round(unit_multiplier * zkcrypto_original_cpu_2[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_ORIGINAL_4_CORES', str(round(unit_multiplier * zkcrypto_original_cpu_4[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_ORIGINAL_8_CORES', str(round(unit_multiplier * zkcrypto_original_cpu_8[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_ORIGINAL_16_CORES', str(round(unit_multiplier * zkcrypto_original_cpu_16[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_1_CORES', str(round(unit_multiplier * zkcrypto_parallelized_cpu_1[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_2_CORES', str(round(unit_multiplier * zkcrypto_parallelized_cpu_2[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_4_CORES', str(round(unit_multiplier * zkcrypto_parallelized_cpu_4[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_8_CORES', str(round(unit_multiplier * zkcrypto_parallelized_cpu_8[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_PARALLELIZED_16_CORES', str(round(unit_multiplier * zkcrypto_parallelized_cpu_16[index], 3)))
    filedata = filedata.replace('ZKCRYPTO_DELTA_1_CORES', calc_delta(zkcrypto_original_cpu_1[index], zkcrypto_parallelized_cpu_1[index]))
    filedata = filedata.replace('ZKCRYPTO_DELTA_2_CORES', calc_delta(zkcrypto_original_cpu_2[index], zkcrypto_parallelized_cpu_2[index]))
    filedata = filedata.replace('ZKCRYPTO_DELTA_4_CORES', calc_delta(zkcrypto_original_cpu_4[index], zkcrypto_parallelized_cpu_4[index]))
    filedata = filedata.replace('ZKCRYPTO_DELTA_8_CORES', calc_delta(zkcrypto_original_cpu_8[index], zkcrypto_parallelized_cpu_8[index]))
    filedata = filedata.replace('ZKCRYPTO_DELTA_16_CORES', calc_delta(zkcrypto_original_cpu_16[index], zkcrypto_parallelized_cpu_16[index]))
    # blst-from-scratch
    filedata = filedata.replace('BLST_ORIGINAL_1_CORES', str(round(unit_multiplier * blst_from_scratch_original_cpu_1[index], 3)))
    filedata = filedata.replace('BLST_ORIGINAL_2_CORES', str(round(unit_multiplier * blst_from_scratch_original_cpu_2[index], 3)))
    filedata = filedata.replace('BLST_ORIGINAL_4_CORES', str(round(unit_multiplier * blst_from_scratch_original_cpu_4[index], 3)))
    filedata = filedata.replace('BLST_ORIGINAL_8_CORES', str(round(unit_multiplier * blst_from_scratch_original_cpu_8[index], 3)))
    filedata = filedata.replace('BLST_ORIGINAL_16_CORES', str(round(unit_multiplier * blst_from_scratch_original_cpu_16[index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_1_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_1[index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_2_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_2[index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_4_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_4[index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_8_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_8[index], 3)))
    filedata = filedata.replace('BLST_PARALLELIZED_16_CORES', str(round(unit_multiplier * blst_from_scratch_parallelized_cpu_16[index], 3)))
    filedata = filedata.replace('BLST_DELTA_1_CORES', calc_delta(blst_from_scratch_original_cpu_1[index], blst_from_scratch_parallelized_cpu_1[index]))
    filedata = filedata.replace('BLST_DELTA_2_CORES', calc_delta(blst_from_scratch_original_cpu_2[index], blst_from_scratch_parallelized_cpu_2[index]))
    filedata = filedata.replace('BLST_DELTA_4_CORES', calc_delta(blst_from_scratch_original_cpu_4[index], blst_from_scratch_parallelized_cpu_4[index]))
    filedata = filedata.replace('BLST_DELTA_8_CORES', calc_delta(blst_from_scratch_original_cpu_8[index], blst_from_scratch_parallelized_cpu_8[index]))
    filedata = filedata.replace('BLST_DELTA_16_CORES', calc_delta(blst_from_scratch_original_cpu_16[index], blst_from_scratch_parallelized_cpu_16[index]))
    # mcl
    filedata = filedata.replace('MCL_ORIGINAL_1_CORES', str(round(unit_multiplier * mcl_original_cpu_1[index], 3)))
    filedata = filedata.replace('MCL_ORIGINAL_2_CORES', str(round(unit_multiplier * mcl_original_cpu_2[index], 3)))
    filedata = filedata.replace('MCL_ORIGINAL_4_CORES', str(round(unit_multiplier * mcl_original_cpu_4[index], 3)))
    filedata = filedata.replace('MCL_ORIGINAL_8_CORES', str(round(unit_multiplier * mcl_original_cpu_8[index], 3)))
    filedata = filedata.replace('MCL_ORIGINAL_16_CORES', str(round(unit_multiplier * mcl_original_cpu_16[index], 3)))
    filedata = filedata.replace('MCL_PARALLELIZED_1_CORES', str(round(unit_multiplier * mcl_parallelized_cpu_1[index], 3)))
    filedata = filedata.replace('MCL_PARALLELIZED_2_CORES', str(round(unit_multiplier * mcl_parallelized_cpu_2[index], 3)))
    filedata = filedata.replace('MCL_PARALLELIZED_4_CORES', str(round(unit_multiplier * mcl_parallelized_cpu_4[index], 3)))
    filedata = filedata.replace('MCL_PARALLELIZED_8_CORES', str(round(unit_multiplier * mcl_parallelized_cpu_8[index], 3)))
    filedata = filedata.replace('MCL_PARALLELIZED_16_CORES', str(round(unit_multiplier * mcl_parallelized_cpu_16[index], 3)))
    filedata = filedata.replace('MCL_DELTA_1_CORES', calc_delta(mcl_original_cpu_1[index], mcl_parallelized_cpu_1[index]))
    filedata = filedata.replace('MCL_DELTA_2_CORES', calc_delta(mcl_original_cpu_2[index], mcl_parallelized_cpu_2[index]))
    filedata = filedata.replace('MCL_DELTA_4_CORES', calc_delta(mcl_original_cpu_4[index], mcl_parallelized_cpu_4[index]))
    filedata = filedata.replace('MCL_DELTA_8_CORES', calc_delta(mcl_original_cpu_8[index], mcl_parallelized_cpu_8[index]))
    filedata = filedata.replace('MCL_DELTA_16_CORES', calc_delta(mcl_original_cpu_16[index], mcl_parallelized_cpu_16[index]))

    with open(OUTPUT_DIR
              + title
                      .replace('(', '_')
                      .replace(')', '_')
                      .replace('=', '_')
              + "_"
              + category
              + '_table.tex',
              'w') as file:
        file.write(filedata)


def make_other_latex_from_data(title: str, category: str, time_unit: str, max_time_to_override_with: float, rust_kzg_index: int, rust_binding_index: int, go_binding_index: int, extra_index=None):
    unit_multiplier = 1
    if time_unit == 's':
        unit_multiplier = 0.001
    if time_unit == 'ns':
        unit_multiplier = 0.000001

    gen_other_graph(title, category, time_unit, unit_multiplier, max_time_to_override_with, rust_kzg_index, rust_binding_index, go_binding_index, extra_index)
    gen_other_table(title, category, time_unit, unit_multiplier, rust_kzg_index, rust_binding_index, go_binding_index, extra_index)


def make_rust_kzg_latex_from_data(title: str, category: str, time_unit: str, max_time_to_override_with: float, index: int):
    unit_multiplier = 1
    if time_unit == 's':
        unit_multiplier = 0.001

    gen_rust_kzg_graph(title, category, time_unit, unit_multiplier, max_time_to_override_with, index)
    gen_rust_kzg_table(title, category, time_unit, unit_multiplier, index)


'''''

* go-kzg-4844   = [
    [0]  Benchmark/BlobToKZGCommitment
    [1]  Benchmark/ComputeKZGProof
    [2]  Benchmark/ComputeBlobKZGProof
    [3]  Benchmark/VerifyKZGProof
    [4]  Benchmark/VerifyBlobKZGProof
    [5]  Benchmark/VerifyBlobKZGProofBatch(count=1)
    [6]  Benchmark/VerifyBlobKZGProofBatch(count=2)
    [7]  Benchmark/VerifyBlobKZGProofBatch(count=4)
    [8]  Benchmark/VerifyBlobKZGProofBatch(count=8)
    [9]  Benchmark/VerifyBlobKZGProofBatch(count=16)
    [10] Benchmark/VerifyBlobKZGProofBatch(count=32)
    [11] Benchmark/VerifyBlobKZGProofBatch(count=64)
    [12] Benchmark/VerifyBlobKZGProofBatchPar(count=1)
    [13] Benchmark/VerifyBlobKZGProofBatchPar(count=2)
    [14] Benchmark/VerifyBlobKZGProofBatchPar(count=4)
    [15] Benchmark/VerifyBlobKZGProofBatchPar(count=8)
    [16] Benchmark/VerifyBlobKZGProofBatchPar(count=16)
    [17] Benchmark/VerifyBlobKZGProofBatchPar(count=32)
    [18] Benchmark/VerifyBlobKZGProofBatchPar(count=64)
]

* rust-kzg      = [
    [0]  bench_DAS_extension scale: '15'
    [1]  blob_to_kzg_commitment
    [2]  compute_kzg_proof
    [3]  verify_kzg_proof
    [4]  compute_blob_kzg_proof
    [5]  verify_blob_kzg_proof
    [6]  verify_blob_kzg_proof_batch/1
    [7]  verify_blob_kzg_proof_batch/2
    [8]  verify_blob_kzg_proof_batch/4
    [9]  verify_blob_kzg_proof_batch/8
    [10] verify_blob_kzg_proof_batch/16
    [11] verify_blob_kzg_proof_batch/32
    [12] verify_blob_kzg_proof_batch/64
    [13] bench_fft_fr scale: '15'
    [14] bench_fft_g1 scale: '15'
    [15] bench_commit_to_poly scale: '15'
    [16] bench_compute_proof_single scale: '15'
    [17] bench_g1_lincomb points: '4096'
    [18] bench_new_poly_div scale: '15'
    [19] bench_recover scale: '15'
    [20] bench_zero_poly scale: '15'
]

* rust-binding  = [
    [0]  blob_to_kzg_commitment
    [1]  compute_kzg_proof
    [2]  compute_blob_kzg_proof
    [3]  verify_kzg_proof
    [4]  verify_blob_kzg_proof
    [5]  verify_blob_kzg_proof_batch/1
    [6]  verify_blob_kzg_proof_batch/2
    [7]  verify_blob_kzg_proof_batch/4
    [8]  verify_blob_kzg_proof_batch/8
    [9]  verify_blob_kzg_proof_batch/16
    [10] verify_blob_kzg_proof_batch/32
    [11] verify_blob_kzg_proof_batch/64
]

* go-binding    = [
    [0]  Benchmark/BlobToKZGCommitment-16
    [1]  Benchmark/ComputeKZGProof-16
    [2]  Benchmark/ComputeBlobKZGProof-16
    [3]  Benchmark/VerifyKZGProof-16
    [4]  Benchmark/VerifyBlobKZGProof-16
    [5]  Benchmark/VerifyBlobKZGProofBatch(count=1)-16
    [6]  Benchmark/VerifyBlobKZGProofBatch(count=2)-16
    [7]  Benchmark/VerifyBlobKZGProofBatch(count=4)-16
    [8]  Benchmark/VerifyBlobKZGProofBatch(count=8)-16
    [9]  Benchmark/VerifyBlobKZGProofBatch(count=16)-16
    [10] Benchmark/VerifyBlobKZGProofBatch(count=32)-16
    [11] Benchmark/VerifyBlobKZGProofBatch(count=64)-16
]

'''''


def main():
    # TODO: add optional step to preprocess the file
    read_bench_file(BENCH_FILE)

    '''''
    Print parsed results
    '''''

    print("------------------------------------ cpu 1 results -------------------------------------")
    print("go_kzg_4844_cpu_1                     = {}".format(go_kzg_4844_cpu_1))
    print("c_kzg_4844_rust_binding_cpu_1         = {}".format(c_kzg_4844_rust_binding_cpu_1))
    print("arkworks_original_cpu_1               = {}".format(arkworks_original_cpu_1))
    print("arkworks_parallelized_cpu_1           = {}".format(arkworks_parallelized_cpu_1))
    print("zkcrypto_original_cpu_1               = {}".format(zkcrypto_original_cpu_1))
    print("zkcrypto_parallelized_cpu_1           = {}".format(zkcrypto_parallelized_cpu_1))
    print("blst_from_scratch_original_cpu_1      = {}".format(blst_from_scratch_original_cpu_1))
    print("blst_from_scratch_parallelized_cpu_1  = {}".format(blst_from_scratch_parallelized_cpu_1))
    print("mcl_original_cpu_1                    = {}".format(mcl_original_cpu_1))
    print("mcl_parallelized_cpu_1                = {}".format(mcl_parallelized_cpu_1))
    print("blst_from_scratch_rust_binding_cpu_1  = {}".format(blst_from_scratch_rust_binding_cpu_1))
    print("blst_from_scratch_go_binding_cpu_1    = {}".format(blst_from_scratch_go_binding_cpu_1))
    print("------------------------------------ cpu 2 results -------------------------------------")
    print("go_kzg_4844_cpu_2                     = {}".format(go_kzg_4844_cpu_2))
    print("c_kzg_4844_rust_binding_cpu_2         = {}".format(c_kzg_4844_rust_binding_cpu_2))
    print("arkworks_original_cpu_2               = {}".format(arkworks_original_cpu_2))
    print("arkworks_parallelized_cpu_2           = {}".format(arkworks_parallelized_cpu_2))
    print("zkcrypto_original_cpu_2               = {}".format(zkcrypto_original_cpu_2))
    print("zkcrypto_parallelized_cpu_2           = {}".format(zkcrypto_parallelized_cpu_2))
    print("blst_from_scratch_original_cpu_2      = {}".format(blst_from_scratch_original_cpu_2))
    print("blst_from_scratch_parallelized_cpu_2  = {}".format(blst_from_scratch_parallelized_cpu_2))
    print("mcl_original_cpu_2                    = {}".format(mcl_original_cpu_2))
    print("mcl_parallelized_cpu_2                = {}".format(mcl_parallelized_cpu_2))
    print("blst_from_scratch_rust_binding_cpu_2  = {}".format(blst_from_scratch_rust_binding_cpu_2))
    print("blst_from_scratch_go_binding_cpu_2    = {}".format(blst_from_scratch_go_binding_cpu_2))
    print("------------------------------------ cpu 4 results -------------------------------------")
    print("go_kzg_4844_cpu_4                     = {}".format(go_kzg_4844_cpu_4))
    print("c_kzg_4844_rust_binding_cpu_4         = {}".format(c_kzg_4844_rust_binding_cpu_4))
    print("arkworks_original_cpu_4               = {}".format(arkworks_original_cpu_4))
    print("arkworks_parallelized_cpu_4           = {}".format(arkworks_parallelized_cpu_4))
    print("zkcrypto_original_cpu_4               = {}".format(zkcrypto_original_cpu_4))
    print("zkcrypto_parallelized_cpu_4           = {}".format(zkcrypto_parallelized_cpu_4))
    print("blst_from_scratch_original_cpu_4      = {}".format(blst_from_scratch_original_cpu_4))
    print("blst_from_scratch_parallelized_cpu_4  = {}".format(blst_from_scratch_parallelized_cpu_4))
    print("mcl_original_cpu_4                    = {}".format(mcl_original_cpu_4))
    print("mcl_parallelized_cpu_4                = {}".format(mcl_parallelized_cpu_4))
    print("blst_from_scratch_rust_binding_cpu_4  = {}".format(blst_from_scratch_rust_binding_cpu_4))
    print("blst_from_scratch_go_binding_cpu_4    = {}".format(blst_from_scratch_go_binding_cpu_4))
    print("------------------------------------ cpu 8 results -------------------------------------")
    print("go_kzg_4844_cpu_8                     = {}".format(go_kzg_4844_cpu_8))
    print("c_kzg_4844_rust_binding_cpu_8         = {}".format(c_kzg_4844_rust_binding_cpu_8))
    print("arkworks_original_cpu_8               = {}".format(arkworks_original_cpu_8))
    print("arkworks_parallelized_cpu_8           = {}".format(arkworks_parallelized_cpu_8))
    print("zkcrypto_original_cpu_8               = {}".format(zkcrypto_original_cpu_8))
    print("zkcrypto_parallelized_cpu_8           = {}".format(zkcrypto_parallelized_cpu_8))
    print("blst_from_scratch_original_cpu_8      = {}".format(blst_from_scratch_original_cpu_8))
    print("blst_from_scratch_parallelized_cpu_8  = {}".format(blst_from_scratch_parallelized_cpu_8))
    print("mcl_original_cpu_8                    = {}".format(mcl_original_cpu_8))
    print("mcl_parallelized_cpu_8                = {}".format(mcl_parallelized_cpu_8))
    print("blst_from_scratch_rust_binding_cpu_8  = {}".format(blst_from_scratch_rust_binding_cpu_8))
    print("blst_from_scratch_go_binding_cpu_8    = {}".format(blst_from_scratch_go_binding_cpu_8))
    print("------------------------------------ cpu 16 results ------------------------------------")
    print("go_kzg_4844_cpu_16                    = {}".format(go_kzg_4844_cpu_16))
    print("c_kzg_4844_rust_binding_cpu_16        = {}".format(c_kzg_4844_rust_binding_cpu_16))
    print("arkworks_original_cpu_16              = {}".format(arkworks_original_cpu_16))
    print("arkworks_parallelized_cpu_16          = {}".format(arkworks_parallelized_cpu_16))
    print("zkcrypto_original_cpu_16              = {}".format(zkcrypto_original_cpu_16))
    print("zkcrypto_parallelized_cpu_16          = {}".format(zkcrypto_parallelized_cpu_16))
    print("blst_from_scratch_original_cpu_16     = {}".format(blst_from_scratch_original_cpu_16))
    print("blst_from_scratch_parallelized_cpu_16 = {}".format(blst_from_scratch_parallelized_cpu_16))
    print("mcl_original_cpu_16                   = {}".format(mcl_original_cpu_16))
    print("mcl_parallelized_cpu_16               = {}".format(mcl_parallelized_cpu_16))
    print("blst_from_scratch_rust_binding_cpu_16 = {}".format(blst_from_scratch_rust_binding_cpu_16))
    print("blst_from_scratch_go_binding_cpu_16   = {}".format(blst_from_scratch_go_binding_cpu_16))

    '''''
    Check parsed results
    '''''

    # --------------------------- cpu 1 ---------------------------
    check_go_kzg_condition(go_kzg_4844_cpu_1)
    check_other_condition(c_kzg_4844_rust_binding_cpu_1)
    check_rust_kzg_condition(arkworks_original_cpu_1)
    check_rust_kzg_condition(arkworks_parallelized_cpu_1)
    check_rust_kzg_condition(zkcrypto_original_cpu_1)
    check_rust_kzg_condition(zkcrypto_parallelized_cpu_1)
    check_rust_kzg_condition(blst_from_scratch_original_cpu_1)
    check_rust_kzg_condition(blst_from_scratch_parallelized_cpu_1)
    check_rust_kzg_condition(mcl_original_cpu_1)
    check_rust_kzg_condition(mcl_parallelized_cpu_1)
    check_other_condition(blst_from_scratch_rust_binding_cpu_1)
    check_other_condition(blst_from_scratch_go_binding_cpu_1)
    # --------------------------- cpu 2 ---------------------------
    check_go_kzg_condition(go_kzg_4844_cpu_2)
    check_other_condition(c_kzg_4844_rust_binding_cpu_2)
    check_rust_kzg_condition(arkworks_original_cpu_2)
    check_rust_kzg_condition(arkworks_parallelized_cpu_2)
    check_rust_kzg_condition(zkcrypto_original_cpu_2)
    check_rust_kzg_condition(zkcrypto_parallelized_cpu_2)
    check_rust_kzg_condition(blst_from_scratch_original_cpu_2)
    check_rust_kzg_condition(blst_from_scratch_parallelized_cpu_2)
    check_rust_kzg_condition(mcl_original_cpu_2)
    check_rust_kzg_condition(mcl_parallelized_cpu_2)
    check_other_condition(blst_from_scratch_rust_binding_cpu_2)
    check_other_condition(blst_from_scratch_go_binding_cpu_2)
    # --------------------------- cpu 4 ---------------------------
    check_go_kzg_condition(go_kzg_4844_cpu_4)
    check_other_condition(c_kzg_4844_rust_binding_cpu_4)
    check_rust_kzg_condition(arkworks_original_cpu_4)
    check_rust_kzg_condition(arkworks_parallelized_cpu_4)
    check_rust_kzg_condition(zkcrypto_original_cpu_4)
    check_rust_kzg_condition(zkcrypto_parallelized_cpu_4)
    check_rust_kzg_condition(blst_from_scratch_original_cpu_4)
    check_rust_kzg_condition(blst_from_scratch_parallelized_cpu_4)
    check_rust_kzg_condition(mcl_original_cpu_4)
    check_rust_kzg_condition(mcl_parallelized_cpu_4)
    check_other_condition(blst_from_scratch_rust_binding_cpu_4)
    check_other_condition(blst_from_scratch_go_binding_cpu_4)
    # --------------------------- cpu 8 ---------------------------
    check_go_kzg_condition(go_kzg_4844_cpu_8)
    check_other_condition(c_kzg_4844_rust_binding_cpu_8)
    check_rust_kzg_condition(arkworks_original_cpu_8)
    check_rust_kzg_condition(arkworks_parallelized_cpu_8)
    check_rust_kzg_condition(zkcrypto_original_cpu_8)
    check_rust_kzg_condition(zkcrypto_parallelized_cpu_8)
    check_rust_kzg_condition(blst_from_scratch_original_cpu_8)
    check_rust_kzg_condition(blst_from_scratch_parallelized_cpu_8)
    check_rust_kzg_condition(mcl_original_cpu_8)
    check_rust_kzg_condition(mcl_parallelized_cpu_8)
    check_other_condition(blst_from_scratch_rust_binding_cpu_8)
    check_other_condition(blst_from_scratch_go_binding_cpu_8)
    # --------------------------- cpu 16 ---------------------------
    check_go_kzg_condition(go_kzg_4844_cpu_16)
    check_other_condition(c_kzg_4844_rust_binding_cpu_16)
    check_rust_kzg_condition(arkworks_original_cpu_16)
    check_rust_kzg_condition(arkworks_parallelized_cpu_16)
    check_rust_kzg_condition(zkcrypto_original_cpu_16)
    check_rust_kzg_condition(zkcrypto_parallelized_cpu_16)
    check_rust_kzg_condition(blst_from_scratch_original_cpu_16)
    check_rust_kzg_condition(blst_from_scratch_parallelized_cpu_16)
    check_rust_kzg_condition(mcl_original_cpu_16)
    check_rust_kzg_condition(mcl_parallelized_cpu_16)
    check_other_condition(blst_from_scratch_rust_binding_cpu_16)
    check_other_condition(blst_from_scratch_go_binding_cpu_16)

    '''''
    rust-kzg (EIP-4844)
    '''''

    make_rust_kzg_latex_from_data("blob_to_kzg_commitment", "EIP4844", "ms", 0,                 index=1)
    make_rust_kzg_latex_from_data("compute_kzg_proof", "EIP4844", "ms", 0,                      index=2)
    make_rust_kzg_latex_from_data("verify_kzg_proof", "EIP4844", "ms", 8,                       index=3)
    make_rust_kzg_latex_from_data("compute_blob_kzg_proof", "EIP4844", "ms", 0,                 index=4)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof", "EIP4844", "ms", 10,                 index=5)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=1)", "EIP4844", "ms", 10, index=6)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=2)", "EIP4844", "ms", 0,  index=7)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=4)", "EIP4844", "ms", 0,  index=8)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=8)", "EIP4844", "ms", 0,  index=9)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=16)", "EIP4844", "ms", 0, index=10)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=32)", "EIP4844", "ms", 0, index=11)
    make_rust_kzg_latex_from_data("verify_blob_kzg_proof_batch_(count=64)", "EIP4844", "ms", 0, index=12)

    '''''
    rust-kzg (KZG)
    '''''

    make_rust_kzg_latex_from_data("das_extension", "KZG", "ms", 0,        index=0)
    make_rust_kzg_latex_from_data("fft_fr", "KZG", "ms", 0,               index=13)
    make_rust_kzg_latex_from_data("fft_g1", "KZG", "s", 0,                index=14)
    make_rust_kzg_latex_from_data("commit_to_poly", "KZG", "ms", 0,       index=15)
    make_rust_kzg_latex_from_data("compute_proof_single", "KZG", "ms", 0, index=16)
    make_rust_kzg_latex_from_data("verify_kzg_proof", "KZG", "ms", 8,     index=3)
    make_rust_kzg_latex_from_data("g1_lincomb", "KZG", "ms", 0,           index=17)
    make_rust_kzg_latex_from_data("new_poly_div", "KZG", "ms", 0,         index=18)
    make_rust_kzg_latex_from_data("recover", "KZG", "ms", 0,              index=19)
    make_rust_kzg_latex_from_data("zero_poly", "KZG", "ms", 0,            index=20)

    '''''
    other (BINDINGS)
    '''''

    make_other_latex_from_data("blob_to_kzg_commitment", "BINDINGS", "ms", 0,                  rust_kzg_index=1,  rust_binding_index=0,  go_binding_index=0)
    make_other_latex_from_data("compute_kzg_proof", "BINDINGS", "ms", 0,                       rust_kzg_index=2,  rust_binding_index=1,  go_binding_index=1)
    make_other_latex_from_data("verify_kzg_proof", "BINDINGS", "ms", 4,                        rust_kzg_index=3,  rust_binding_index=3,  go_binding_index=3)
    make_other_latex_from_data("compute_blob_kzg_proof", "BINDINGS", "ms", 0,                  rust_kzg_index=4,  rust_binding_index=2,  go_binding_index=2)
    make_other_latex_from_data("verify_blob_kzg_proof", "BINDINGS", "ms", 6.5,                 rust_kzg_index=5,  rust_binding_index=4,  go_binding_index=4)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=1)", "BINDINGS", "ms", 6.5, rust_kzg_index=6,  rust_binding_index=5,  go_binding_index=5,  extra_index=12)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=2)", "BINDINGS", "ms", 10,  rust_kzg_index=7,  rust_binding_index=6,  go_binding_index=6,  extra_index=13)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=4)", "BINDINGS", "ms", 0,   rust_kzg_index=8,  rust_binding_index=7,  go_binding_index=7,  extra_index=14)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=8)", "BINDINGS", "ms", 0,   rust_kzg_index=9,  rust_binding_index=8,  go_binding_index=8,  extra_index=15)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=16)", "BINDINGS", "ms", 0,  rust_kzg_index=10, rust_binding_index=9,  go_binding_index=9,  extra_index=16)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=32)", "BINDINGS", "ms", 0,  rust_kzg_index=11, rust_binding_index=10, go_binding_index=10, extra_index=17)
    make_other_latex_from_data("verify_blob_kzg_proof_batch_(count=64)", "BINDINGS", "ms", 0,  rust_kzg_index=12, rust_binding_index=11, go_binding_index=11, extra_index=18)


if __name__ == '__main__':
    main()
