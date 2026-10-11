/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::c_char;
use std::os::raw::c_void;

use crate::{
    SERVO_COOKIE_SAME_SITE_LAX, SERVO_COOKIE_SAME_SITE_NONE, SERVO_COOKIE_SAME_SITE_STRICT,
    ServoCookie, ServoSiteDataManager,
};
use cookie::Cookie;
use url::Url;

/// Cookie visitor callback.
///
/// Called for each cookie during enumeration.
///
/// `cookie` - The cookie being visited (valid only for the duration of the callback).
/// `count` - The 0-based index of the current cookie.
/// `total` - The total number of cookies that will be visited.
/// `user_data` - The user data pointer passed to the visit function.
///
/// Returns `true` to continue enumeration, `false` to stop.
pub type ServoCookieVisitor = Option<
    unsafe extern "C" fn(
        cookie: *const ServoCookie,
        count: usize,
        total: usize,
        user_data: *mut c_void,
    ) -> bool,
>;

/// Cookie deletion callback.
///
/// Called when a cookie deletion operation completes.
///
/// `num_deleted` - The number of cookies deleted.
/// `user_data` - The user data pointer passed to the delete function.
pub type ServoDeleteCookieCallback =
    Option<unsafe extern "C" fn(num_deleted: usize, user_data: *mut c_void)>;

/// Flush/store callback.
///
/// Called when a flush/store operation completes.
///
/// `user_data` - The user data pointer passed to the flush function.
pub type ServoFlushStoreCallback = Option<unsafe extern "C" fn(user_data: *mut c_void)>;

/// Cookie filter for deletion operations.
///
/// All fields are optional filters. A NULL pointer means "match all".
/// The `secure` and `session` fields are tri-state: use the
/// `ServoCookieFilterFlags` to indicate which ones are active.
#[repr(C)]
pub struct ServoCookieFilter {
    /// Filter by URL (domain + path). NULL = match all URLs.
    pub url: *const c_char,
    /// Filter by cookie name. NULL = match all names.
    pub name: *const c_char,
    /// Filter by domain. NULL = match all domains.
    pub domain: *const c_char,
    /// Filter by path. NULL = match all paths.
    pub path: *const c_char,
    /// Filter by secure flag. Only meaningful if `flags` has
    /// `SERVO_COOKIE_FILTER_HAS_SECURE` set.
    pub secure: bool,
    /// Filter by session flag. Only meaningful if `flags` has
    /// `SERVO_COOKIE_FILTER_HAS_SESSION` set.
    pub session: bool,
    /// Bitflags indicating which optional fields are active.
    /// See `ServoCookieFilterFlags`.
    pub flags: u32,
}

/// Flags for `ServoCookieFilter`.
///
/// These indicate which optional filter fields are active.
#[repr(u32)]
pub enum ServoCookieFilterFlags {
    /// No flags set.
    None = 0,
    /// The `secure` field is active (filter by secure vs non-secure).
    HasSecure = 1 << 0,
    /// The `session` field is active (filter by session vs persistent).
    HasSession = 1 << 1,
}

/// Internal storage for cookie strings returned to C.
/// We store owned strings here and return pointers to them.
struct CookieStringStorage {
    names: Vec<String>,
    values: Vec<String>,
    domains: Vec<String>,
    paths: Vec<String>,
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
        // Store all strings and get their stable pointers
        self.names.push(cookie.name().to_string());
        self.values.push(cookie.value().to_string());
        self.domains.push(cookie.domain().unwrap_or("").to_string());
        self.paths.push(cookie.path().unwrap_or("/").to_string());

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
            name: self.names[idx].as_ptr() as *const c_char,
            value: self.values[idx].as_ptr() as *const c_char,
            domain: self.domains[idx].as_ptr() as *const c_char,
            path: self.paths[idx].as_ptr() as *const c_char,
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

/// Helper to call visitor for each cookie.
fn visit_cookies_with_callback(
    cookies: Vec<Cookie<'static>>,
    visitor: ServoCookieVisitor,
    user_data: *mut c_void,
) -> bool {
    let Some(visitor) = visitor else {
        return true;
    };

    COOKIE_STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        storage.clear();

        let total = cookies.len();
        let mut stored_cookies = Vec::with_capacity(total);
        for cookie in &cookies {
            stored_cookies.push(storage.store(cookie));
        }

        for (count, servo_cookie) in stored_cookies.iter().enumerate() {
            // SAFETY: visitor is a valid function pointer, servo_cookie is valid for this call
            let continue_visit = unsafe { visitor(servo_cookie, count, total, user_data) };
            if !continue_visit {
                return false;
            }
        }
        true
    })
}

/// Visits all cookies in the cookie store.
///
/// `manager` - The site data manager handle.
/// `visitor` - The callback to invoke for each cookie.
/// `user_data` - Opaque user data passed to the visitor.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - If `visitor` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_visit_all_cookies(
    manager: *mut ServoSiteDataManager,
    visitor: ServoCookieVisitor,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }
        if visitor.is_none() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };

        // Get all cookies from SiteDataManager
        let cookies = manager.servo.site_data_manager().visit_all_cookies();

        // Call visitor for each cookie
        visit_cookies_with_callback(cookies, visitor, user_data);

        crate::ServoError::Ok
    })
}

/// Visits all cookies for a given URL.
///
/// `manager` - The site data manager handle.
/// `url` - The URL to filter cookies for (NUL-terminated UTF-8 string).
/// `visitor` - The callback to invoke for each cookie.
/// `user_data` - Opaque user data passed to the visitor.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - `url` is a non-null pointer to a valid C string.
/// - If `visitor` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_visit_cookies_for_url(
    manager: *mut ServoSiteDataManager,
    url: *const c_char,
    visitor: ServoCookieVisitor,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }
        if url.is_null() {
            return crate::ServoError::NullArgument;
        }
        if visitor.is_none() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        let url_str = unsafe { std::ffi::CStr::from_ptr(url) }
            .to_str()
            .map_err(|_| crate::ServoError::InvalidUtf8)?;

        let url = Url::parse(url_str).map_err(|_| crate::ServoError::InvalidUtf8)?;

        // Get cookies for URL from SiteDataManager
        let cookies = manager
            .servo
            .site_data_manager()
            .visit_cookies_for_url(url, servo_api::CookieSource::HTTP);

        // Call visitor for each cookie
        visit_cookies_with_callback(cookies, visitor, user_data);

        crate::ServoError::Ok
    })
}

/// Deletes all cookies matching the given filter.
///
/// `manager` - The site data manager handle.
/// `filter` - The filter to match cookies for deletion. Can be NULL to delete all cookies.
/// `callback` - Optional callback invoked when deletion completes.
/// `user_data` - Opaque user data passed to the callback.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - If `filter` is not NULL, it points to a valid `ServoCookieFilter` struct.
/// - If `callback` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_delete_all_cookies(
    manager: *mut ServoSiteDataManager,
    filter: *const ServoCookieFilter,
    callback: ServoDeleteCookieCallback,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };

        // Parse filter if provided
        let (url_opt, name_opt, domain_opt, path_opt, secure_opt, session_opt) = if filter.is_null()
        {
            (None, None, None, None, None, None)
        } else {
            let filter = unsafe { &*filter };
            let url = if filter.url.is_null() {
                None
            } else {
                let url_str = unsafe { std::ffi::CStr::from_ptr(filter.url) }
                    .to_str()
                    .map_err(|_| crate::ServoError::InvalidUtf8)?;
                Some(Url::parse(url_str).map_err(|_| crate::ServoError::InvalidUtf8)?)
            };
            let name = if filter.name.is_null() || (filter.flags & 0) != 0 {
                None
            } else {
                let name_str = unsafe { std::ffi::CStr::from_ptr(filter.name) }
                    .to_str()
                    .map_err(|_| crate::ServoError::InvalidUtf8)?;
                Some(name_str.to_string())
            };
            let domain = if filter.domain.is_null() {
                None
            } else {
                let domain_str = unsafe { std::ffi::CStr::from_ptr(filter.domain) }
                    .to_str()
                    .map_err(|_| crate::ServoError::InvalidUtf8)?;
                Some(domain_str.to_string())
            };
            let path = if filter.path.is_null() {
                None
            } else {
                let path_str = unsafe { std::ffi::CStr::from_ptr(filter.path) }
                    .to_str()
                    .map_err(|_| crate::ServoError::InvalidUtf8)?;
                Some(path_str.to_string())
            };
            let secure = if filter.flags & ServoCookieFilterFlags::HasSecure as u32 != 0 {
                Some(filter.secure)
            } else {
                None
            };
            let session = if filter.flags & ServoCookieFilterFlags::HasSession as u32 != 0 {
                Some(filter.session)
            } else {
                None
            };
            (url, name, domain, path, secure, session)
        };

        // Call delete_all_cookies on SiteDataManager
        manager.servo.site_data_manager().delete_all_cookies(
            url_opt,
            name_opt,
            domain_opt,
            path_opt,
            secure_opt,
            session_opt,
            callback.map(|cb| {
                let user_data = user_data;
                Box::new(move |count: usize| {
                    unsafe { cb(count, user_data) };
                })
            }),
        );

        crate::ServoError::Ok
    })
}

/// Deletes all cookies for a given URL.
///
/// `manager` - The site data manager handle.
/// `url` - The URL to delete cookies for (NUL-terminated UTF-8 string).
/// `callback` - Optional callback invoked when deletion completes.
/// `user_data` - Opaque user data passed to the callback.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - `url` is a non-null pointer to a valid C string.
/// - If `callback` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_delete_cookies_for_url(
    manager: *mut ServoSiteDataManager,
    url: *const c_char,
    callback: ServoDeleteCookieCallback,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }
        if url.is_null() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        let url_str = unsafe { std::ffi::CStr::from_ptr(url) }
            .to_str()
            .map_err(|_| crate::ServoError::InvalidUtf8)?;

        let url = Url::parse(url_str).map_err(|_| crate::ServoError::InvalidUtf8)?;

        // Call delete_cookies_for_url on SiteDataManager
        manager.servo.site_data_manager().delete_cookies_for_url(
            url,
            callback.map(|cb| {
                let user_data = user_data;
                Box::new(move |count: usize| {
                    unsafe { cb(count, user_data) };
                })
            }),
        );

        crate::ServoError::Ok
    })
}

/// Deletes a specific cookie by name for a given URL.
///
/// `manager` - The site data manager handle.
/// `url` - The URL the cookie is associated with (NUL-terminated UTF-8 string).
/// `name` - The name of the cookie to delete (NUL-terminated UTF-8 string).
/// `callback` - Optional callback invoked when deletion completes.
/// `user_data` - Opaque user data passed to the callback.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - `url` is a non-null pointer to a valid C string.
/// - `name` is a non-null pointer to a valid C string.
/// - If `callback` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_delete_cookie(
    manager: *mut ServoSiteDataManager,
    url: *const c_char,
    name: *const c_char,
    callback: ServoDeleteCookieCallback,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }
        if url.is_null() {
            return crate::ServoError::NullArgument;
        }
        if name.is_null() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };
        let url_str = unsafe { std::ffi::CStr::from_ptr(url) }
            .to_str()
            .map_err(|_| crate::ServoError::InvalidUtf8)?;
        let name_str = unsafe { std::ffi::CStr::from_ptr(name) }
            .to_str()
            .map_err(|_| crate::ServoError::InvalidUtf8)?;

        let url = Url::parse(url_str).map_err(|_| crate::ServoError::InvalidUtf8)?;

        // Call delete_cookie on SiteDataManager
        manager.servo.site_data_manager().delete_cookie(
            url,
            name_str.to_string(),
            callback.map(|cb| {
                let user_data = user_data;
                Box::new(move |count: usize| {
                    unsafe { cb(count, user_data) };
                })
            }),
        );

        crate::ServoError::Ok
    })
}

/// Flushes the cookie store to disk (persists all cookies).
///
/// `manager` - The site data manager handle.
/// `callback` - Optional callback invoked when flush completes.
/// `user_data` - Opaque user data passed to the callback.
///
/// Returns `ServoError::Ok` on success, or an error code on failure.
///
/// # Safety
///
/// The caller must ensure that:
/// - `manager` is a valid `ServoSiteDataManager` from `servo_get_site_data_manager`.
/// - If `callback` is not NULL, it must be a valid function pointer.
/// - The call is made from the same thread that created the `Servo` instance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn servo_site_data_flush_store(
    manager: *mut ServoSiteDataManager,
    callback: ServoFlushStoreCallback,
    user_data: *mut c_void,
) -> crate::ServoError {
    crate::ServoError::catch_ffi(|| {
        if manager.is_null() {
            return crate::ServoError::NullArgument;
        }

        // SAFETY: The caller is assumed to uphold the safety requirements.
        let manager = unsafe { &*manager };

        // Call flush_store on SiteDataManager
        manager
            .servo
            .site_data_manager()
            .flush_store(callback.map(|cb| {
                let user_data = user_data;
                Box::new(move || {
                    unsafe { cb(user_data) };
                })
            }));

        crate::ServoError::Ok
    })
}
