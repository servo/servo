/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::{c_char, c_void, CString};
use std::os::raw::c_int;

use cookie::Cookie;
use servo_api::Servo;
use url::Url;

use crate::ServoError;

/// SameSite attribute values for cookies.
/// Corresponds to `cookie::SameSite` variants.
pub const SERVO_COOKIE_SAME_SITE_LAX: c_int = 0;
pub const SERVO_COOKIE_SAME_SITE_STRICT: c_int = 1;
pub const SERVO_COOKIE_SAME_SITE_NONE: c_int = 2;

/// Opaque handle for the SiteDataManager.
///
/// Obtained via `servo_get_site_data_manager(servo)`.
/// The handle is tied to the `Servo` instance lifetime and must not be used
/// after `servo_free` is called.
///
/// # Thread safety
///
/// The handle must be used only from the thread that created the `Servo` instance.
// cbindgen:opaque
pub struct ServoSiteDataManager {
    pub(crate) servo: Servo,
}

/// Cookie struct for C ABI compatibility.
///
/// The string pointers (`name`, `value`, `domain`, `path`) point to memory
/// owned by Servo and are valid until the next call to any
/// `servo_site_data_*` function. The embedder must copy them if longer
/// lifetime is needed.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ServoCookie {
    pub name: *const c_char,
    pub value: *const c_char,
    pub domain: *const c_char,
    pub path: *const c_char,
    /// Unix timestamp in seconds. 0 = session cookie.
    pub expires: i64,
    pub secure: bool,
    pub http_only: bool,
    /// One of SERVO_COOKIE_SAME_SITE_* constants.
    pub same_site: c_int,
}

/// Internal storage for cookie strings returned to C.
/// We store owned strings here and return pointers to them.
struct CookieStringStorage {
    names: Vec<CString>,
    values: Vec<CString>,
    domains: Vec<CString>,
    paths: Vec<CString>,
}

impl CookieStringStorage {
    fn new() -> Self {
        Self {
            names: Vec::new(),
            values: Vec::new(),
            domains: Vec::new(),
            paths: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.names.clear();
        self.values.clear();
        self.domains.clear();
        self.paths.clear();
    }

    fn store(&mut self, cookie: &Cookie<'static>) -> ServoCookie {
        // Store all strings as null-terminated CStrings and get their stable pointers
        self.names.push(CString::new(cookie.name()).unwrap_or_default());
        self.values.push(CString::new(cookie.value()).unwrap_or_default());
        self.domains.push(CString::new(cookie.domain().unwrap_or("")).unwrap_or_default());
        self.paths.push(CString::new(cookie.path().unwrap_or("/")).unwrap_or_default());

        let idx = self.names.len() - 1;

        // Convert expiration to Unix timestamp
        let expires = match cookie.expires() {
            Some(cookie::Expiration::DateTime(dt)) => dt.unix_timestamp(),
            Some(cookie::Expiration::Session) | None => 0,
        };

        // Convert SameSite to our C constant
        let same_site = match cookie.same_site() {
            Some(cookie::SameSite::Lax) => SERVO_COOKIE_SAME_SITE_LAX,
            Some(cookie::SameSite::Strict) => SERVO_COOKIE_SAME_SITE_STRICT,
            Some(cookie::SameSite::None) => SERVO_COOKIE_SAME_SITE_NONE,
            None => SERVO_COOKIE_SAME_SITE_LAX, // Default to Lax if not set
        };

        ServoCookie {
            name: self.names[idx].as_ptr(),
            value: self.values[idx].as_ptr(),
            domain: self.domains[idx].as_ptr(),
            path: self.paths[idx].as_ptr(),
            expires,
            secure: cookie.secure().unwrap_or(false),
            http_only: cookie.http_only().unwrap_or(false),
            same_site,
        }
    }
}

// Thread-local storage for cookie strings.
// This is safe because the C API is single-threaded (same thread as Servo).
thread_local! {
    static COOKIE_STORAGE: std::cell::RefCell<CookieStringStorage> =
        std::cell::RefCell::new(CookieStringStorage::new());
}

/// Gets the SiteDataManager handle from a Servo instance.
///
/// `servo` is a handle to a `Servo` object.
/// The ownership of `servo` remains with the caller after the call.
///
/// Returns a newly allocated `ServoSiteDataManager` handle. The ownership of
/// the returned handle is transferred to the caller, who must free it with
/// [`servo_site_data_manager_free`]. The returned handle is valid as long as
/// `servo` is valid.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `servo` is a non-null pointer to a `Servo` instance previously
///   returned by `servo_builder_build` and has not yet been freed nor
///   passed to another API that takes ownership of it.
/// - The call is made from the same thread that originally created the
///   `Servo` instance via `servo_builder_build`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_get_site_data_manager(
    servo: *mut Servo,
) -> *mut ServoSiteDataManager {
    ServoError::catch_ffi_ptr(|| {
        if servo.is_null() {
            return std::ptr::null_mut();
        }

        // SAFETY: The caller is assumed to uphold the safety requirements for
        // `servo` documented above.
        let servo = unsafe { &*servo }.clone();

        Box::into_raw(Box::new(ServoSiteDataManager { servo }))
    })
}

/// Frees the `ServoSiteDataManager` handle.
///
/// `manager` is a handle to a `ServoSiteDataManager` object.
/// The ownership of `manager` is transferred to the function. The caller
/// must not use or free `manager` again.
///
/// # Safety
///
/// The caller must ensure that `manager` was previously returned by
/// `servo_get_site_data_manager` and has not yet been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_manager_free(manager: *mut ServoSiteDataManager) {
    if manager.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: The caller is assumed to uphold the safety requirements.
        unsafe {
            let _ = Box::from_raw(manager);
        }
    }));
}

/// Gets cookies for a given URL.
///
/// This is a buffer-query function: call with `buffer = NULL` and `buffer_len = 0`
/// to get the required number of cookies, then allocate a buffer of that size
/// and call again to fill it.
///
/// `manager` is a handle to a `ServoSiteDataManager` object (from
/// `servo_get_site_data_manager`). The ownership remains with the caller.
///
/// `url` is a NUL-terminated UTF-8 string representing the URL to get cookies for.
///
/// `buffer` is a pointer to a caller-allocated array of `ServoCookie` structs.
/// If `NULL`, the function returns the number of cookies that would be written.
///
/// `buffer_len` is the number of `ServoCookie` elements the buffer can hold.
///
/// Returns the number of cookies that would be written (including those that
/// didn't fit if `buffer_len` is too small). If `buffer` is non-NULL and
/// `buffer_len` is sufficient, the cookies are written into the buffer.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - `url` is a non-null pointer to a valid C string that remains unmodified
///   for the duration of the call.
/// - If `buffer` is non-NULL, it points to at least `buffer_len * sizeof(ServoCookie)`
///   writable bytes.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_cookies_for_url(
    manager: *mut ServoSiteDataManager,
    url: *const c_char,
    buffer: *mut ServoCookie,
    buffer_len: usize,
) -> usize {
    if manager.is_null() || url.is_null() {
        return 0;
    }

    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        let url_str = match unsafe { std::ffi::CStr::from_ptr(url) }.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return 0,
        };

        // Get cookies from SiteDataManager
        let cookies = manager
            .servo
            .site_data_manager()
            .cookies_for_url(url, servo_api::CookieSource::HTTP);

        // Store strings in thread-local storage
        COOKIE_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.clear();

            let count = cookies.len();
            let write_count = std::cmp::min(count, buffer_len);

            // Store all cookies first (we need to keep all strings alive)
            let mut stored_cookies = Vec::with_capacity(count);
            for cookie in &cookies {
                stored_cookies.push(storage.store(cookie));
            }

            // Copy to buffer if provided
            if !buffer.is_null() && buffer_len > 0 {
                for i in 0..write_count {
                    // SAFETY: buffer points to valid memory for at least buffer_len elements
                    unsafe {
                        *buffer.add(i) = stored_cookies[i];
                    }
                }
            }

            count
        })
    }))
    .unwrap_or(0)
}

/// Sets a cookie for a given URL.
///
/// `manager` is a handle to a `ServoSiteDataManager` object.
/// The ownership remains with the caller.
///
/// `url` is a NUL-terminated UTF-8 string representing the URL to set the cookie for.
///
/// `cookie` is a pointer to a `ServoCookie` struct with the cookie data.
/// The string pointers in the cookie must be valid NUL-terminated UTF-8 strings.
/// The `expires` field is a Unix timestamp in seconds (0 = session cookie).
/// The `same_site` field must be one of the SERVO_COOKIE_SAME_SITE_* constants.
///
/// Returns 0 on success, -1 on error (e.g., invalid URL or cookie data).
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - `url` is a non-null pointer to a valid C string that remains unmodified
///   for the duration of the call.
/// - `cookie` is a non-null pointer to a valid `ServoCookie` struct.
/// - The string pointers in `cookie` are valid NUL-terminated UTF-8 strings.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_set_cookie(
    manager: *mut ServoSiteDataManager,
    url: *const c_char,
    cookie: *const ServoCookie,
) -> c_int {
    if manager.is_null() || url.is_null() || cookie.is_null() {
        return -1;
    }

    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        let url_str = match unsafe { std::ffi::CStr::from_ptr(url) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return -1,
        };

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let cookie_ref = unsafe { &*cookie };
        if cookie_ref.name.is_null()
            || cookie_ref.value.is_null()
            || cookie_ref.domain.is_null()
            || cookie_ref.path.is_null()
        {
            return -1;
        }

        // Validate and extract cookie data
        let name = match unsafe { std::ffi::CStr::from_ptr(cookie_ref.name) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let value = match unsafe { std::ffi::CStr::from_ptr(cookie_ref.value) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let domain = match unsafe { std::ffi::CStr::from_ptr(cookie_ref.domain) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        let path = match unsafe { std::ffi::CStr::from_ptr(cookie_ref.path) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };

        // Build the cookie
        let mut rust_cookie = Cookie::new(name, value);

        if !domain.is_empty() {
            rust_cookie.set_domain(domain);
        }
        if !path.is_empty() {
            rust_cookie.set_path(path);
        }

        // Handle expiration
        if cookie_ref.expires > 0 {
            use cookie::time::OffsetDateTime;
            match OffsetDateTime::from_unix_timestamp(cookie_ref.expires) {
                Ok(dt) => rust_cookie.set_expires(dt),
                Err(_) => return -1,
            }
        } else {
            rust_cookie.set_expires(None); // Session cookie
        }

        rust_cookie.set_secure(cookie_ref.secure);
        rust_cookie.set_http_only(cookie_ref.http_only);

        // Handle SameSite
        let same_site = match cookie_ref.same_site {
            SERVO_COOKIE_SAME_SITE_LAX => cookie::SameSite::Lax,
            SERVO_COOKIE_SAME_SITE_STRICT => cookie::SameSite::Strict,
            SERVO_COOKIE_SAME_SITE_NONE => cookie::SameSite::None,
            _ => return -1, // Invalid SameSite value
        };
        rust_cookie.set_same_site(same_site);

        // Call SiteDataManager to set the cookie (synchronous)
        manager
            .servo
            .site_data_manager()
            .set_cookie_for_url(url, rust_cookie.into_owned(), None);

        0
    }))
    .unwrap_or(-1)
}

/// Clears all cookies (both persistent and session) from both public and
/// private cookie jars.
///
/// `manager` is a handle to a `ServoSiteDataManager` object.
/// The ownership remains with the caller.
///
/// This is a synchronous operation.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_clear_cookies(manager: *mut ServoSiteDataManager) {
    if manager.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        manager.servo.site_data_manager().clear_cookies(None);
    }));
}

/// Clears all session cookies (cookies with no expiry/max-age) from both
/// public and private cookie jars.
///
/// `manager` is a handle to a `ServoSiteDataManager` object.
/// The ownership remains with the caller.
///
/// This is a synchronous operation.
///
/// # Safety
///
/// The caller must ensure that:
///
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_clear_session_cookies(manager: *mut ServoSiteDataManager) {
    if manager.is_null() {
        return;
    }

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        manager
            .servo
            .site_data_manager()
            .clear_session_cookies(None);
    }));
}

/// Storage types for site data enumeration and clearing.
/// Corresponds to `servo_api::StorageType` bitflags.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServoStorageType {
    /// HTTP cookies
    Cookies = 1 << 0,
    /// `localStorage` Web API
    Local = 1 << 1,
    /// `sessionStorage` Web API
    Session = 1 << 2,
    /// All storage types (Cookies | Local | Session)
    All = (1 << 0) | (1 << 1) | (1 << 2),
}

/// Represents site data for a single site/domain.
///
/// The string pointers (`name`, `site`) point to NUL-terminated strings owned
/// by Servo and are valid only for the duration of the visitor callback.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ServoSiteData {
    /// The site/domain name (eTLD+1).
    pub name: *const c_char,
    /// Alias for `name` pointing to the same NUL-terminated string.
    pub site: *const c_char,
    /// Bitmask of `ServoStorageType` flags present for this site.
    pub storage_types: u32,
}

/// Visitor callback for site data enumeration.
///
/// `site` - The site data entry (valid only for the duration of the callback).
/// `count` - 0-based index of current entry.
/// `total` - Total number of sites being enumerated.
/// `user_data` - Opaque user pointer passed to `servo_site_data_visit_sites`.
///
/// Return `true` to continue enumeration, or `false` to stop early.
pub type ServoSitesVisitor = Option<
    unsafe extern "C" fn(
        site: *const ServoSiteData,
        count: usize,
        total: usize,
        user_data: *mut c_void,
    ) -> bool,
>;

pub type ServoSiteDataVisitor = ServoSitesVisitor;

/// Enumerate sites and their stored data types.
///
/// Calls `visitor` for each site that has stored data matching `storage_types`.
///
/// `manager` is a valid `ServoSiteDataManager` handle.
/// `storage_types` is a bitmask of `ServoStorageType` flags to filter by.
/// `visitor` is the callback invoked for each matching site.
/// `user_data` is an opaque pointer forwarded to `visitor`.
///
/// Returns `ServoError::Ok` on success, or an error code.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` handle from `servo_get_site_data_manager`.
/// - If `visitor` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_visit_sites(
    manager: *mut ServoSiteDataManager,
    storage_types: u32,
    visitor: ServoSitesVisitor,
    user_data: *mut c_void,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if manager.is_null() {
            return ServoError::NullArgument;
        }
        let Some(visitor) = visitor else {
            return ServoError::NullArgument;
        };

        let manager = unsafe { &*manager };

        let mut rust_storage_types = servo_api::StorageType::empty();
        if (storage_types & (ServoStorageType::Cookies as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Cookies;
        }
        if (storage_types & (ServoStorageType::Local as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Local;
        }
        if (storage_types & (ServoStorageType::Session as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Session;
        }

        let sites = manager.servo.site_data_manager().site_data(rust_storage_types);
        let total = sites.len();

        for (count, site_entry) in sites.into_iter().enumerate() {
            let Ok(c_name) = CString::new(site_entry.name()) else {
                continue;
            };

            let mut c_types = 0u32;
            let st = site_entry.storage_types();
            if st.contains(servo_api::StorageType::Cookies) {
                c_types |= ServoStorageType::Cookies as u32;
            }
            if st.contains(servo_api::StorageType::Local) {
                c_types |= ServoStorageType::Local as u32;
            }
            if st.contains(servo_api::StorageType::Session) {
                c_types |= ServoStorageType::Session as u32;
            }

            let entry = ServoSiteData {
                name: c_name.as_ptr(),
                site: c_name.as_ptr(),
                storage_types: c_types,
            };

            let continue_visit = unsafe { visitor(&entry, count, total, user_data) };
            if !continue_visit {
                break;
            }
        }

        ServoError::Ok
    })
}

/// Clear site data for the given sites and storage types.
///
/// `manager` is a valid `ServoSiteDataManager` handle.
/// `sites` is an array of NUL-terminated site strings (eTLD+1).
/// `count` is the number of site strings in `sites`.
/// `storage_types` is a bitmask of `ServoStorageType` flags specifying which data to clear.
///
/// Returns `ServoError::Ok` on success, or an error code.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` handle from `servo_get_site_data_manager`.
/// - If `count > 0`, `sites` points to an array of at least `count` non-null valid C strings.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_clear_site_data(
    manager: *mut ServoSiteDataManager,
    sites: *const *const c_char,
    count: usize,
    storage_types: u32,
) -> ServoError {
    ServoError::catch_ffi(|| {
        if manager.is_null() {
            return ServoError::NullArgument;
        }
        if count > 0 && sites.is_null() {
            return ServoError::NullArgument;
        }

        let manager = unsafe { &*manager };

        let mut rust_storage_types = servo_api::StorageType::empty();
        if (storage_types & (ServoStorageType::Cookies as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Cookies;
        }
        if (storage_types & (ServoStorageType::Local as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Local;
        }
        if (storage_types & (ServoStorageType::Session as u32)) != 0 {
            rust_storage_types |= servo_api::StorageType::Session;
        }

        let mut site_strings = Vec::with_capacity(count);
        for i in 0..count {
            let site_ptr = unsafe { *sites.add(i) };
            if site_ptr.is_null() {
                return ServoError::NullArgument;
            }
            let site_str = match unsafe { std::ffi::CStr::from_ptr(site_ptr) }.to_str() {
                Ok(s) => s,
                Err(_) => return ServoError::InvalidUtf8,
            };
            site_strings.push(site_str);
        }

        manager
            .servo
            .site_data_manager()
            .clear_site_data(&site_strings, rust_storage_types);

        ServoError::Ok
    })
}

// Re-export cookie manager types and functions
pub use crate::cookie_manager::{
    ServoCookieFilter, ServoCookieFilterFlags, ServoCookieVisitor, ServoDeleteCookieCallback,
    ServoFlushStoreCallback,
};
