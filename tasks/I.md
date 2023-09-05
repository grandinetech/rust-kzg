Task I - Upgrade
  
* Become a member of one team ("arkworks", "blst", "mcl", "zkcrypto"), team sizes should be roughly equal;
* Get familiar with [rust-kzg](https://github.com/sifraitech/rust-kzg), especially your team's crate;
* The goal of the task is to upgrade the codebase until the latest [c-kzg-4844](https://github.com/ethereum/c-kzg-4844). Currently, [rust-kzg](https://github.com/sifraitech/rust-kzg) uses an old version [5703f6f3536b7692616bc289ac3f3867ab8db9d8](https://github.com/sifraitech/rust-kzg/blob/main/.github/workflows/blst-benchmarks.yml#L6C25-L6C65) of [c-kzg-4844](https://github.com/ethereum/c-kzg-4844);
* Teams need to upgrade dependencies in their chosen crates (bump versions in `Cargo.toml` files);
* The work will likely need a creative branching strategy because the entire toolchain (benchmarks etc.) is shared between the crates. One option would be for the fastest team to ship an updated toolchain and then other teams upgrade their crates.
* The build must pass on Github CI.
 
Points: 2

Deadline 2023-10-03

----------------------------------------------------------------

Each task has its deadline specified in the task that allows getting 100% of the points if done correctly. However, each late week significantly reduces the points:

1 week -25%
2 week -50%
3 week -75%
4 week -100%
