// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

template <class T>
struct host_pinned_allocator_t {
    typedef T value_type;
    host_pinned_allocator_t() {}

    // A converting copy constructor:
    template<class U>
    host_pinned_allocator_t(const host_pinned_allocator_t<U>&) {}
    
    template<class U>
    bool operator==(const host_pinned_allocator_t<U>&) const {
        return true;
    }
    
    template<class U>
    bool operator!=(const host_pinned_allocator_t<U>&) const {
        return false;
    }
    
    T* allocate(const size_t n) const {
        if (n == 0) {
            return nullptr;
        }
        if (n > static_cast<size_t>(-1) / sizeof(T)) {
            throw std::bad_array_new_length();
        }
        void *pv = NULL;
        cudaMallocHost(&pv, n * sizeof(T));
        if (!pv) {
            throw std::bad_alloc();
        }
        return static_cast<T*>(pv);
    }
    void deallocate(T* const p, size_t) const {
        cudaFreeHost(p);
    }        
};


