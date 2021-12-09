use cstr_core::CString;

/// Open shared memory object with given name and size.
///
/// # Arguments
/// * `name` - name of the shared memory object
/// * `size` - size of the shared memory object
///
/// # Returns
/// * `SharedMemory` - shared memory object
fn open(name: &str, size: usize) -> i32 {
    let c_name = CString::new(name).unwrap();
    let shm_fd = unsafe {
        libc::shm_open(
            c_name.as_ptr(),
            libc::O_CREAT | libc::O_RDWR,
            libc::S_IRUSR | libc::S_IWUSR,
        )
    };
    if shm_fd < 0 {
        log::error!("shm_open failed");
        return -1;
    }

    let shm_size = unsafe { libc::ftruncate(shm_fd, size as i64) };
    if shm_size < 0 {
        log::error!("ftruncate failed");
        return -1;
    }

    shm_fd
}

/// Mmap shared memory object with given file descriptor.
///
/// # Arguments
/// * `fd` - file descriptor
/// * `size` - size of the shared memory object
///
/// # Returns
/// * pointer to the shared memory object
fn map(fd: i32, size: usize) -> *mut libc::c_void {
    let ptr = unsafe {
        libc::mmap(
            core::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };

    if ptr == libc::MAP_FAILED {
        log::error!("mmap failed");
        return core::ptr::null_mut();
    }

    ptr
}

/// Create a new shared memory object with given name and size.
///
/// # Arguments
/// * `name` - name of the shared memory object
/// * `size` - size of the shared memory object
///
/// # Returns
/// * `ptr` - pointer to the shared memory object
pub fn create_shm(name: &str, size: usize) -> *mut libc::c_void {
    let fd = open(name, size);

    map(fd, size)
}

#[allow(dead_code)]
/// Remove the shared memory object with given name.
///
/// # Arguments
/// * `name` - name of the shared memory object
pub fn destroy_shm(name: &str) {
    let c_name = CString::new(name).unwrap();
    let shm_fd = unsafe { libc::shm_unlink(c_name.as_ptr()) };
    if shm_fd < 0 {
        log::error!("shm_unlink failed");
    }
}
