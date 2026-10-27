/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Host → ArkTS notification callbacks.
//!
//! This module only *adds* callback slots on top of the three `OnceLock` slots that already
//! live in [`super`]: no pre-existing slot, function body or call site is modified, so the
//! whole file can be upstreamed (or dropped) without disturbing the current embedder.
//!
//! Every slot is re-registerable (`LazyLock<Mutex<Option<…>>>`) so that an ability that is
//! recreated can install a fresh thread-safe function instead of keeping a closure that
//! points at a destroyed page.

use std::sync::{LazyLock, Mutex};

use log::debug;
use napi_derive_ohos::napi;
use napi_ohos::bindgen_prelude::Function;
use napi_ohos::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use servo::{Cursor, LoadStatus};

/// Queue length for the thread-safe functions that push small, ordered state updates
/// (title, navigation state, load status, cursor, shortcut) to ArkTS.
const STATE_CALLBACK_QUEUE_SIZE: usize = 1;
/// Queue length for the thread-safe functions that push larger, lossy payloads
/// (favicon, download URL) to ArkTS.
const PAYLOAD_CALLBACK_QUEUE_SIZE: usize = 4;

type StringStateCallback = ThreadsafeFunction<
    String,
    (),
    String,
    napi_ohos::Status,
    false,
    false,
    STATE_CALLBACK_QUEUE_SIZE,
>;
type U32StateCallback =
    ThreadsafeFunction<u32, (), u32, napi_ohos::Status, false, false, STATE_CALLBACK_QUEUE_SIZE>;
type StringPayloadCallback = ThreadsafeFunction<
    String,
    (),
    String,
    napi_ohos::Status,
    false,
    false,
    PAYLOAD_CALLBACK_QUEUE_SIZE,
>;

/// Title of the currently focused webview, used for the ArkTS tab labels.
static SET_TITLE_CB: LazyLock<Mutex<Option<StringStateCallback>>> =
    LazyLock::new(|| Mutex::new(None));
/// Navigation state for the ArkTS toolbar: bit 0 = can go back, bit 1 = can go forward.
static NAV_STATE_CB: LazyLock<Mutex<Option<U32StateCallback>>> = LazyLock::new(|| Mutex::new(None));
/// Page load progress: 1 = started, 2 = head parsed, 3 = complete.
static LOAD_STATUS_CB: LazyLock<Mutex<Option<U32StateCallback>>> =
    LazyLock::new(|| Mutex::new(None));
/// Cursor updates for the focused webview.
static CURSOR_CB: LazyLock<Mutex<Option<U32StateCallback>>> = LazyLock::new(|| Mutex::new(None));
/// Last cursor id pushed to ArkTS, to avoid redundant callbacks on every mouse move.
static LAST_CURSOR: Mutex<u32> = Mutex::new(u32::MAX);
/// 站点图标（favicon）回调：payload 为 "宽,高:base64(RGBA)"。
static FAVICON_CB: LazyLock<Mutex<Option<StringPayloadCallback>>> =
    LazyLock::new(|| Mutex::new(None));
/// 内核观测到的附件下载地址（Content-Disposition: attachment）。
static DOWNLOAD_SINK_CB: LazyLock<Mutex<Option<StringPayloadCallback>>> =
    LazyLock::new(|| Mutex::new(None));
/// Browser keyboard shortcuts, forwarded to the ArkTS chrome as small integer ids.
static SHORTCUT_CB: LazyLock<Mutex<Option<U32StateCallback>>> = LazyLock::new(|| Mutex::new(None));

/// Maps servo's CSS cursor onto a small integer id that ArkTS turns into a `PointerStyle`.
fn cursor_id(cursor: Cursor) -> u32 {
    match cursor {
        Cursor::Default => 0,
        Cursor::Pointer => 1,
        Cursor::Text | Cursor::VerticalText => 2,
        Cursor::Crosshair => 3,
        Cursor::Move => 4,
        Cursor::Grab => 5,
        Cursor::Grabbing => 6,
        Cursor::NotAllowed | Cursor::NoDrop => 7,
        Cursor::Wait => 8,
        Cursor::Progress => 9,
        Cursor::ContextMenu => 10,
        Cursor::Help => 11,
        Cursor::Copy | Cursor::Alias => 12,
        Cursor::Cell => 13,
        Cursor::EResize | Cursor::WResize | Cursor::EwResize => 14,
        Cursor::NResize | Cursor::SResize | Cursor::NsResize => 15,
        Cursor::NeResize | Cursor::SwResize | Cursor::NeswResize => 16,
        Cursor::NwResize | Cursor::SeResize | Cursor::NwseResize => 17,
        _ => 0,
    }
}

/// Pushes the page's CSS cursor to ArkTS, but only when it actually changed.
pub(crate) fn notify_cursor(cursor: Cursor) {
    let id = cursor_id(cursor);
    {
        let mut last = LAST_CURSOR.lock().unwrap();
        if *last == id {
            return;
        }
        *last = id;
    }
    if let Some(callback) = CURSOR_CB.lock().unwrap().as_ref() {
        let _ = callback.call(id, ThreadsafeFunctionCallMode::NonBlocking);
    }
}

pub(crate) fn notify_title(title: Option<String>) {
    let text = title.unwrap_or_default();
    if let Some(callback) = SET_TITLE_CB.lock().unwrap().as_ref() {
        let _ = callback.call(text, ThreadsafeFunctionCallMode::NonBlocking);
    }
}

/// 来自 `WebViewDelegate` 的导航状态（这是 servo 0.7 唯一有效的事件通道）。
pub(crate) fn notify_nav_state(can_back: bool, can_forward: bool) {
    let state = u32::from(can_back) | (u32::from(can_forward) << 1);
    if let Some(callback) = NAV_STATE_CB.lock().unwrap().as_ref() {
        let _ = callback.call(state, ThreadsafeFunctionCallMode::NonBlocking);
    }
}

pub(crate) fn notify_load_status(status: LoadStatus) {
    let code = if status == LoadStatus::Started {
        1
    } else if status == LoadStatus::HeadParsed {
        2
    } else {
        3
    };
    if let Some(callback) = LOAD_STATUS_CB.lock().unwrap().as_ref() {
        let _ = callback.call(code, ThreadsafeFunctionCallMode::NonBlocking);
    }
}

#[napi(js_name = "registerTitleCallback")]
pub fn register_title_callback(callback: Function<String, ()>) -> napi_ohos::Result<()> {
    debug!("register_title_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<STATE_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *SET_TITLE_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerNavStateCallback")]
pub fn register_nav_state_callback(callback: Function<u32, ()>) -> napi_ohos::Result<()> {
    debug!("register_nav_state_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<STATE_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *NAV_STATE_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerLoadStatusCallback")]
pub fn register_load_status_callback(callback: Function<u32, ()>) -> napi_ohos::Result<()> {
    debug!("register_load_status_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<STATE_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *LOAD_STATUS_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerCursorCallback")]
pub fn register_cursor_callback(callback: Function<u32, ()>) -> napi_ohos::Result<()> {
    debug!("register_cursor_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<STATE_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *CURSOR_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerShortcutCallback")]
pub fn register_shortcut_callback(callback: Function<u32, ()>) -> napi_ohos::Result<()> {
    debug!("register_shortcut_callback called!");
    let tsfn_builder = callback.build_threadsafe_function();
    let function = tsfn_builder
        .max_queue_size::<STATE_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *SHORTCUT_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerFaviconCallback")]
pub fn register_favicon_callback(callback: Function<String, ()>) -> napi_ohos::Result<()> {
    let function = callback
        .build_threadsafe_function()
        .max_queue_size::<PAYLOAD_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *FAVICON_CB.lock().unwrap() = Some(function);
    Ok(())
}

#[napi(js_name = "registerDownloadSink")]
pub fn register_download_sink(callback: Function<String, ()>) -> napi_ohos::Result<()> {
    let function = callback
        .build_threadsafe_function()
        .max_queue_size::<PAYLOAD_CALLBACK_QUEUE_SIZE>()
        .build()?;
    *DOWNLOAD_SINK_CB.lock().unwrap() = Some(function);
    Ok(())
}
