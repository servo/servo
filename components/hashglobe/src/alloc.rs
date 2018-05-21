// FORK NOTE: Copied from liballoc_system, removed unnecessary APIs,
// APIs take size/align directly instead of Layout




// The minimum alignment guaranteed by the architecture. This value is used to
// add fast paths for low alignment values. In practice, the alignment is a
// constant at the call site and the branch will be optimized out.
#[cfg(all(any(target_arch = "x86",
              target_arch = "arm",
              target_arch = "mips",
              target_arch = "powerpc",
              target_arch = "powerpc64",
              target_arch = "asmjs",
              target_arch = "wasm32")))]
const MIN_ALIGN: usize = 8;
#[cfg(all(any(target_arch = "x86_64",
              target_arch = "aarch64",
              target_arch = "mips64",
              target_arch = "s390x",
              target_arch = "sparc64")))]
const MIN_ALIGN: usize = 16;

pub use self::platform::{alloc, dealloc, realloc};

#[cfg(any(unix, target_os = "redox"))]
mod platform {
    extern crate libc;

    #[cfg(not(any(target_os = "android")))]
    use std::ptr;

    use super::MIN_ALIGN;

    #[inline]
    pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
        let ptr = if align <= MIN_ALIGN {
            libc::malloc(size) as *mut u8
        } else {
            aligned_malloc(size, align)
        };
        ptr
    }

    #[inline]
    pub unsafe fn dealloc(ptr: *mut u8, _align: usize) {
        libc::free(ptr as *mut libc::c_void)
    }

    #[inline]
    pub unsafe fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
        libc::realloc(ptr as *mut libc::c_void, new_size) as *mut u8
    }

    #[cfg(any(target_os = "android", target_os = "redox"))]
    #[inline]
    unsafe fn aligned_malloc(size: usize, align: usize) -> *mut u8 {
        // On android we currently target API level 9 which unfortunately
        // doesn't have the `posix_memalign` API used below. Instead we use
        // `memalign`, but this unfortunately has the property on some systems
        // where the memory returned cannot be deallocated by `free`!
        //
        // Upon closer inspection, however, this appears to work just fine with
        // Android, so for this platform we should be fine to call `memalign`
        // (which is present in API level 9). Some helpful references could
        // possibly be chromium using memalign [1], attempts at documenting that
        // memalign + free is ok [2] [3], or the current source of chromium
        // which still uses memalign on android [4].
        //
        // [1]: https://codereview.chromium.org/10796020/
        // [2]: https://code.google.com/p/android/issues/detail?id=35391
        // [3]: https://bugs.chromium.org/p/chromium/issues/detail?id=138579
        // [4]: https://chromium.googlesource.com/chromium/src/base/+/master/
        //                                       /memory/aligned_memory.cc
        libc::memalign(align, size) as *mut u8
    }

    #[cfg(not(any(target_os = "android", target_os = "redox")))]
    #[inline]
    unsafe fn aligned_malloc(size: usize, align: usize) -> *mut u8 {
        let mut out = ptr::null_mut();
        let ret = libc::posix_memalign(&mut out, align, size);
        if ret != 0 {
            ptr::null_mut()
        } else {
            out as *mut u8
        }
    }
}

#[cfg(windows)]
#[allow(bad_style)]
mod platform {

    use super::MIN_ALIGN;
    type LPVOID = *mut u8;
    type HANDLE = LPVOID;
    type SIZE_T = usize;
    type DWORD = u32;
    type BOOL = i32;


    extern "system" {
        fn GetProcessHeap() -> HANDLE;
        fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
        fn HeapReAlloc(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID, dwBytes: SIZE_T) -> LPVOID;
        fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
        fn GetLastError() -> DWORD;
    }

    #[repr(C)]
    struct Header(*mut u8);

    unsafe fn get_header<'a>(ptr: *mut u8) -> &'a mut Header {
        &mut *(ptr as *mut Header).offset(-1)
    }

    unsafe fn align_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
        let aligned = ptr.offset((align - (ptr as usize & (align - 1))) as isize);
        *get_header(aligned) = Header(ptr);
        aligned
    }

    #[inline]
    unsafe fn allocate_with_flags(size: usize, align: usize, flags: DWORD) -> *mut u8
    {
        if align <= MIN_ALIGN {
            HeapAlloc(GetProcessHeap(), flags, size)
        } else {
            let size = size + align;
            let ptr = HeapAlloc(GetProcessHeap(), flags, size);
            if ptr.is_null() {
                ptr
            } else {
                align_ptr(ptr, align)
            }
        }
    }

    #[inline]
    pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
        allocate_with_flags(size, align, 0)
    }

    #[inline]
    pub unsafe fn dealloc(ptr: *mut u8, align: usize) {
        if align <= MIN_ALIGN {
            let err = HeapFree(GetProcessHeap(), 0, ptr as LPVOID);
            debug_assert!(err != 0, "Failed to free heap memory: {}",
                          GetLastError());
        } else {
            let header = get_header(ptr);
            let err = HeapFree(GetProcessHeap(), 0, header.0 as LPVOID);
            debug_assert!(err != 0, "Failed to free heap memory: {}",
                          GetLastError());
        }
    }

    #[inline]
    pub unsafe fn realloc(ptr: *mut u8, new_size: usize) -> *mut u8 {
        HeapReAlloc(GetProcessHeap(),
                    0,
                    ptr as LPVOID,
                    new_size) as *mut u8
    }
}
