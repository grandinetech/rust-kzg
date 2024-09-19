Task III - Join MSM optimisations

* The codebase needs to be updated to the latest [c-kzg-4844](https://github.com/ethereum/c-kzg-4844), dependencies also need to be upgraded;
* The goal is to join all previous optimisations into one optimised MSM implementation. The teams need to collaborate in order to make the optimisations compatible between each other.
* Joints MSM optimisations need to be generic (compatible with multiple ECC backend). If not possible - consult with supervisor.
* Optimisations need to be fuzzed against `go-kzg-4844` and `c-kzg-4844` with the [fuzzer](https://github.com/jtraglia/kzg-fuzz).
* The build must pass on Github CI.
 
Points: 2

Deadline 2023-12-21

----------------------------------------------------------------

Each task has its deadline specified in the task that allows getting 100% of the points if done correctly. However, each late week significantly reduces the points:

1 week -25%
2 week -50%
3 week -75%
4 week -100%
