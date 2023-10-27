Task II - Optimise MSM

* The codebase needs to be updated to the latest [c-kzg-4844](https://github.com/ethereum/c-kzg-4844), dependencies also needs to be upgraded;
* KZG10 does a heavy MSM computation when calculating commitment and opening. `rust-kzg` with `blst` backend uses Pippenger method (including the parallel version of it) and produces decent results, however, `go-kzg-4844` is faster here, so there is a place for improvements. Possible optimisation directions:
  - BGMW algoritm ([this dissertation](https://uwspace.uwaterloo.ca/bitstream/handle/10012/19626/Luo_Guiwen.pdf?sequence=3) is an easy to read source). We will likely benefit from it because our trusted setup is not very large, we can store precomputations in the memory. C++ implementation can be found [here](https://github.com/LuoGuiwen/MSM_blst/blob/2e098f09f07969ac3191406976be6d1c197100f2/main_p1.cpp#L294). However, we need a parallelized Rust version, so it could be that the best way is to build on top of [blst parallel implementation](https://github.com/supranational/blst/blob/master/bindings/rust/src/pippenger.rs#L116). Feel free to explore other code bases and use it if there is actually a more convenient parallel Pippenger implementation in Rust.
  - Other optimisations proposed in that [this dissertation](https://uwspace.uwaterloo.ca/bitstream/handle/10012/19626/Luo_Guiwen.pdf?sequence=3);
  - Optimisations implemented in [arkmsm](https://github.com/snarkify/arkmsm), they also have an [explanation](https://hackmd.io/@drouyang/msm);
  - Cuda GPU parallel algorithms, some examples [here](https://github.com/z-prize/2022-entries/tree/main/open-division/prize1-msm/prize1a-msm-gpu), but very likely there are faster versions now;
  - Discuss other ideas with the supervisor.
* Teams will need to try to figure out a way to get their algorithms useful for other ECC backends, if this is not possible for some reason - MSM optimistations will be needed to be implemented for `blst` backend;  
* The build must pass on Github CI.
 
Points: 2

Deadline 2023-11-09

----------------------------------------------------------------

Each task has its deadline specified in the task that allows getting 100% of the points if done correctly. However, each late week significantly reduces the points:

1 week -25%
2 week -50%
3 week -75%
4 week -100%
