pub trait ThreadPoolExt {
    fn joined_execute<'any, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'any;
}

use core::mem::transmute;
use std::sync::{Mutex, Once};
use threadpool::ThreadPool;

pub fn da_pool() -> ThreadPool {
    static INIT: Once = Once::new();
    static mut POOL: *const Mutex<ThreadPool> = 0 as *const Mutex<ThreadPool>;

    INIT.call_once(|| {
        let pool = Mutex::new(ThreadPool::default());
        // let pool = Mutex::new(ThreadPool::default());
        unsafe { POOL = transmute(Box::new(pool)) };
    });
    unsafe { (*POOL).lock().unwrap().clone() }
}

type Thunk<'any> = Box<dyn FnOnce() + Send + 'any>;

impl ThreadPoolExt for ThreadPool {
    fn joined_execute<'scope, F>(&self, job: F)
    where
        F: FnOnce() + Send + 'scope,
    {
        // Bypass 'lifetime limitations by brute force. It works,
        // because we explicitly join the threads...
        self.execute(unsafe { transmute::<Thunk<'scope>, Thunk<'static>>(Box::new(job)) })
    }
}
