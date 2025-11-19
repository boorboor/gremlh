use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default, Debug)]
pub struct ScanStats {
    total_files: AtomicUsize,
    files_with_gremlins: AtomicUsize,
    total_gremlins: AtomicUsize,
    errors: AtomicUsize,
}

impl ScanStats {
    pub fn inc_total_files(&self) {
        self.total_files.fetch_add(1, Ordering::Relaxed);
    }
    pub fn inc_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }
    pub fn add_gremlins(&self, count: usize) {
        if count > 0 {
            self.files_with_gremlins.fetch_add(1, Ordering::Relaxed);
            self.total_gremlins.fetch_add(count, Ordering::Relaxed);
        }
    }
    pub fn get_total_gremlins(&self) -> usize {
        self.total_gremlins.load(Ordering::Relaxed)
    }
    pub fn get_files_with_gremlins(&self) -> usize {
        self.files_with_gremlins.load(Ordering::Relaxed)
    }
    pub fn get_errors(&self) -> usize {
        self.errors.load(Ordering::Relaxed)
    }
}
