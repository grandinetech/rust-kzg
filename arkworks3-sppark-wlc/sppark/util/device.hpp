// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0


// Class to represent GPU resources
// class devices_t {
// }

template<class T>
class device_ptr_list_t {
    vector<T*> d_ptrs;
public:
    device_ptr_list_t() {}
    ~device_ptr_list_t() {
        for(T *ptr: d_ptrs) {
            cudaFree(ptr);
        }
    }
    size_t allocate(size_t bytes) {
        T *d_ptr;
        CUDA_OK(cudaMalloc(&d_ptr, bytes));
        d_ptrs.push_back(d_ptr);
        return d_ptrs.size() - 1;
    }
    size_t size() {
        return d_ptrs.size();
    }
    // TODO
    // Can set to NULL but can't renumber. Do we need this? 
    // void free(size_t idx) {
    // }
    T* operator[](size_t i) {
        if (i > d_ptrs.size() - 1) {
            CUDA_OK(cudaErrorInvalidDevicePointer);
        }
        return d_ptrs[i];
    }
    
};

// Class to represent a GPU
class device_t {
  
};
