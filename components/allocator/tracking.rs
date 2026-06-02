/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An allocator wrapper that records metadata about each live allocation.
//! This metadata can then be used to identify allocations that are not visible
//! through any existing MallocSizeOf path.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::hash_map::Entry;
use std::io::Write;
use std::os::raw::c_void;
use std::sync::{LazyLock, Mutex};

use rustc_hash::FxHashMap;

/// The maximum number of allocations that we'll keep track of. Once the limit
/// is reached, we'll evict the first allocation that is smaller than the new addition.
const MAX_TRACKED_ALLOCATIONS: usize = usize::MAX;
/// Cap the number of stack frames that we'll store per allocation.
const MAX_FRAMES: usize = 50;
/// A certain number of frames at the top of the allocation stack are just
/// just internal liballoc implementation details or AccountingAlloc functions.
/// We can skip them without losing any meaningful information.
const SKIPPED_FRAMES: usize = 5;
/// The minimum size of allocation that we'll track. Can be used to reduce overhead
/// by skipping bookkeeping for small allocations.
const MIN_SIZE: usize = 0;

thread_local! {
    static IN_ALLOCATION: Cell<bool> = const { Cell::new(false) };
}

#[derive(PartialEq, Eq)]
struct AllocSite {
    /// The stack at the time this allocation was recorded.
    frames: [*mut std::ffi::c_void; MAX_FRAMES],
    /// The start of the allocated memory.
    ptr: *mut u8,
    /// The size of the allocated memory.
    size: usize,
    /// True if this allocation site's size is ever queried after the initial
    /// allocation. If false, it means that the allocation is not visible from
    /// any of the MallocSizeOf roots.
    noted: bool,
}

impl AllocSite {
    fn contains(&self, ptr: *mut u8) -> bool {
        ptr >= self.ptr && ptr < unsafe { self.ptr.add(self.size) }
    }
}

unsafe impl Send for AllocSite {}

/// A map of pointers to allocation callsite metadata.
static ALLOCATION_SITES: LazyLock<Mutex<FxHashMap<usize, AllocSite>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

#[derive(Default)]
pub struct AccountingAlloc<A = System> {
    allocator: A,
}

impl<A> AccountingAlloc<A> {
    pub const fn with_allocator(allocator: A) -> Self {
        Self { allocator }
    }

    #[expect(clippy::absurd_extreme_comparisons)]
    fn remove_allocation(&self, ptr: *const c_void, size: usize) {
        if size < MIN_SIZE {
            return;
        }
        let old = IN_ALLOCATION.with(|status| status.replace(true));
        if old {
            return;
        }
        let mut sites = ALLOCATION_SITES.lock().unwrap();
        if let Entry::Occupied(e) = sites.entry(ptr as usize) {
            e.remove();
        }
        IN_ALLOCATION.with(|status| status.set(old));
    }

    #[expect(clippy::absurd_extreme_comparisons)]
    fn record_allocation(&self, ptr: *mut u8, size: usize) {
        if size < MIN_SIZE {
            return;
        }
        let old = IN_ALLOCATION.with(|status| status.replace(true));
        if old {
            return;
        }
        let mut num_skipped = 0;
        let mut num_frames = 0;
        let mut frames = [std::ptr::null_mut(); MAX_FRAMES];
        backtrace::trace(|frame| {
            if num_skipped < SKIPPED_FRAMES {
                num_skipped += 1;
                return true;
            }
            frames[num_frames] = frame.ip();
            num_frames += 1;
            num_frames < MAX_FRAMES
        });
        let site = AllocSite {
            frames,
            size,
            ptr,
            noted: false,
        };
        let mut sites = ALLOCATION_SITES.lock().unwrap();

        if sites.len() < MAX_TRACKED_ALLOCATIONS {
            sites.insert(ptr as usize, site);
        } else if let Some(key) = sites
            .iter()
            .find(|(_, s)| s.size < site.size)
            .map(|(k, _)| *k)
        {
            sites.remove(&key);
            sites.insert(ptr as usize, site);
        }

        IN_ALLOCATION.with(|status| status.set(old));
    }

    pub(crate) fn enclosing_size(&self, ptr: *const c_void) -> (*const c_void, usize) {
        ALLOCATION_SITES
            .lock()
            .unwrap()
            .iter()
            .find(|(_, site)| site.contains(ptr.cast_mut().cast()))
            .map_or((std::ptr::null_mut(), 0), |(_, site)| {
                (site.ptr.cast(), site.size)
            })
    }

    // The default is an absurd comparisons but you are supposed to change the default.
    #[expect(clippy::absurd_extreme_comparisons)]
    pub(crate) fn note_allocation(&self, ptr: *const c_void, size: usize) {
        if size < MIN_SIZE {
            return;
        }
        IN_ALLOCATION.with(|status| status.set(true));
        if let Some(site) = ALLOCATION_SITES.lock().unwrap().get_mut(&(ptr as usize)) {
            site.noted = true;
        }
        IN_ALLOCATION.with(|status| status.set(false));
    }

    pub(crate) fn dump_unmeasured_allocations(&self, mut writer: impl Write) {
        // Ensure that we ignore all allocations triggered while processing
        // the existing allocation data.
        IN_ALLOCATION.with(|status| status.set(true));
        {
            let sites = ALLOCATION_SITES.lock().unwrap();
            let default = "???";
            for site in sites
                .values()
                .filter(|site| site.size > MIN_SIZE && !site.noted)
            {
                let mut resolved = vec![];
                for ip in site.frames.iter().filter(|ip| !ip.is_null()) {
                    backtrace::resolve(*ip, |symbol| {
                        resolved.push((
                            symbol.filename().map(|f| f.to_owned()),
                            symbol.lineno(),
                            symbol.name().map(|n| format!("{}", n)),
                        ));
                    });
                }

                if let Err(error) = writeln!(&mut writer, "---\n{}\n", site.size) {
                    log::error!("Error writing to log file: {error:?}");
                    return;
                }
                for (filename, line, symbol) in &resolved {
                    let fname = filename.as_ref().map(|f| f.display().to_string());
                    if let Err(error) = writeln!(
                        &mut writer,
                        "{}:{} ({})",
                        fname.as_deref().unwrap_or(default),
                        line.unwrap_or_default(),
                        symbol.as_deref().unwrap_or(default),
                    ) {
                        log::error!("Error writing to log file: {error:?}");
                        return;
                    }
                }
            }
        }
        IN_ALLOCATION.with(|status| status.set(false));
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for AccountingAlloc<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.allocator.alloc(layout) };
        self.record_allocation(ptr, layout.size());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.allocator.dealloc(ptr, layout) };
        self.remove_allocation(ptr.cast(), layout.size());
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.allocator.alloc_zeroed(layout) };
        self.record_allocation(ptr, layout.size());
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.remove_allocation(ptr.cast(), layout.size());
        let ptr = unsafe { self.allocator.realloc(ptr, layout, new_size) };
        self.record_allocation(ptr, new_size);
        ptr
    }
}
