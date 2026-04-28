/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::ffi::c_void;

use libc::pollfd;
use log::{debug, warn};
use ohos_media_sys::avformat::OH_AVFormat;
use ohos_media_sys::avplayer::{
    OH_AVPlayer_Create, OH_AVPlayer_Pause, OH_AVPlayer_Play, OH_AVPlayer_Prepare,
    OH_AVPlayer_Release, OH_AVPlayer_Seek, OH_AVPlayer_SetOnInfoCallback,
    OH_AVPlayer_SetPlaybackSpeed, OH_AVPlayer_SetVideoSurface, OH_AVPlayer_SetVolume,
    OH_AVPlayer_Stop,
};
use ohos_media_sys::avplayer_base::{
    AVPlaybackSpeed, AVPlayerOnInfoType, AVPlayerSeekMode, AVPlayerState, OH_AVPlayer,
};
use ohos_sys_opaque_types::{OH_NativeImage, OHNativeWindow, OHNativeWindowBuffer};
use ohos_window_sys::native_buffer::native_buffer::OH_NativeBuffer_Usage;
use ohos_window_sys::native_image::{
    OH_ConsumerSurface_Create, OH_ConsumerSurface_SetDefaultUsage,
    OH_NativeImage_AcquireNativeWindow, OH_NativeImage_AcquireNativeWindowBuffer,
    OH_NativeImage_Destroy, OH_NativeImage_ReleaseNativeWindowBuffer,
    OH_NativeImage_SetOnFrameAvailableListener, OH_OnFrameAvailableListener,
};
use ohos_window_sys::native_window::{
    OH_NativeWindow_DestroyNativeWindow, OH_NativeWindow_GetBufferHandleFromNative,
    OH_NativeWindow_NativeObjectReference, OH_NativeWindow_NativeObjectUnreference,
};

#[cfg(not(sdk_api_21))]
use crate::ohos_media::dummy_source::MediaSourceWrapper;
#[cfg(sdk_api_21)]
use crate::ohos_media::source::MediaSourceWrapper;

#[repr(C)]
#[derive(Debug)]
pub struct FrameInfo {
    pub fd: i32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub size: i32,
    pub format: i32,
    pub vir_addr: *mut u8,
    native_window_buffer: *mut OHNativeWindowBuffer,
    fence_fd: i32,
}
pub struct OhosPlayer {
    native_image: Option<*mut OH_NativeImage>,
    ohos_av_player: *mut OH_AVPlayer,
    media_data_source: Option<MediaSourceWrapper>,
    event_info_callback_closure: Option<*mut Box<dyn Fn(AVPlayerOnInfoType, *mut OH_AVFormat)>>,
    frame_available_callback_closure: Option<*mut Box<dyn Fn()>>,
    native_window: Option<*mut OHNativeWindow>,
    has_set_source_size: bool,
    has_set_window: bool,
    volume: f64,
    playback_rate: f64,
    state: AVPlayerState,
}

impl OhosPlayer {
    pub fn new() -> Self {
        debug!("Creating OHOS Player!");
        OhosPlayer {
            native_image: None,
            ohos_av_player: unsafe { OH_AVPlayer_Create() },
            media_data_source: None,
            event_info_callback_closure: None,
            frame_available_callback_closure: None,
            native_window: None,
            has_set_source_size: false,
            has_set_window: false,
            volume: 1.0,
            playback_rate: 1.0,
            state: AVPlayerState::AV_IDLE,
        }
    }

    // initialize function
    pub fn set_state(&mut self, state: AVPlayerState) {
        self.state = state;
        self.initialize_check_state_action();
    }
    /// This function would try to run some action during initialize phase
    /// each action should only be run once.
    /// Should try to check whether to run each action when
    /// 1. state changed
    /// 2. after running external initialize step.
    ///    e.g. setup_window_buffer_listener
    pub fn initialize_check_state_action(&mut self) {
        if self.state == AVPlayerState::AV_INITIALIZED &&
            self.native_window.is_some() &&
            !self.has_set_window
        {
            self.has_set_window = true;
            self.set_window_to_player();
            self.prepare(); // only prepare after setting window.
        }
    }

    fn set_window_to_player(&mut self) {
        let Some(native_window) = self.native_window else {
            warn!("Setting window to player, but Native Window not initialized!");
            return;
        };
        unsafe {
            OH_AVPlayer_SetVideoSurface(self.ohos_av_player, native_window);
        }
    }

    /// The first step of initialization process, after this avplayer will
    /// become initialized. Kickstart the initialize process.
    fn setup_data_source(&mut self) {
        let Some(ref mut source) = self.media_data_source else {
            warn!("Error Source not initialized!");
            return;
        };
        debug!("Setting up data source");
        source.set_data_src(self.ohos_av_player);
    }

    pub fn set_volume(&mut self, volume: f64) {
        unsafe {
            OH_AVPlayer_SetVolume(self.ohos_av_player, volume as f32, volume as f32);
        }
        self.volume = volume;
    }

    pub fn volume(&self) -> f64 {
        self.volume
    }

    pub fn set_source(&mut self, source: MediaSourceWrapper) {
        // Todo: Should think of better way to change the way the data is given.
        self.media_data_source = Some(source);
    }

    pub fn end_of_stream(&self) {
        if let Some(source) = &self.media_data_source {
            source.end_of_stream();
        }
    }

    pub fn push_data(&self, data: Vec<u8>) {
        if let Some(inner_source) = &self.media_data_source {
            inner_source.push_data(data);
        }
    }

    pub fn play(&self) {
        unsafe {
            debug!("OH_AVPlayer_Play!");
            OH_AVPlayer_Play(self.ohos_av_player);
        }
    }

    pub fn set_mute(&mut self, mute: bool) {
        debug!("OH_AVPlayer Set mute: {}", mute);
        let volume = match mute {
            true => 0.,
            false => 1.,
        };
        unsafe {
            OH_AVPlayer_SetVolume(self.ohos_av_player, volume, volume);
        }
        self.volume = volume as f64;
    }

    pub fn muted(&self) -> bool {
        self.volume == 0.0
    }

    pub fn seek(&self, second: i32) {
        unsafe {
            log::info!("OH_AVPlayer_Seek! :{}", second);
            OH_AVPlayer_Seek(
                self.ohos_av_player,
                second,
                AVPlayerSeekMode::AV_SEEK_CLOSEST,
            );
        }
    }

    pub fn set_rate(&mut self, rate: f64) {
        self.playback_rate = rate;
        // Round toward 1x: for rates >= 1 round down, for rates < 1 round up.
        let speed = if rate >= 1.0 {
            match rate {
                3.0.. => AVPlaybackSpeed::AV_SPEED_FORWARD_3_00_X,
                2.0.. => AVPlaybackSpeed::AV_SPEED_FORWARD_2_00_X,
                1.75.. => AVPlaybackSpeed::AV_SPEED_FORWARD_1_75_X,
                1.5.. => AVPlaybackSpeed::AV_SPEED_FORWARD_1_50_X,
                1.25.. => AVPlaybackSpeed::AV_SPEED_FORWARD_1_25_X,
                _ => AVPlaybackSpeed::AV_SPEED_FORWARD_1_00_X,
            }
        } else {
            match rate {
                ..=0.0 => AVPlaybackSpeed::AV_SPEED_FORWARD_1_00_X,
                ..=0.125 => AVPlaybackSpeed::AV_SPEED_FORWARD_0_125_X,
                ..=0.25 => AVPlaybackSpeed::AV_SPEED_FORWARD_0_25_X,
                ..=0.5 => AVPlaybackSpeed::AV_SPEED_FORWARD_0_50_X,
                ..=0.75 => AVPlaybackSpeed::AV_SPEED_FORWARD_0_75_X,
                _ => AVPlaybackSpeed::AV_SPEED_FORWARD_1_00_X,
            }
        };
        unsafe {
            OH_AVPlayer_SetPlaybackSpeed(self.ohos_av_player, speed);
        }
    }

    pub fn playback_rate(&self) -> f64 {
        self.playback_rate
    }

    pub fn pause(&self) {
        unsafe {
            debug!("OH_AVPlayer_Pause!");
            OH_AVPlayer_Pause(self.ohos_av_player);
        }
    }

    pub fn stop(&self) {
        unsafe {
            debug!("OH_AVPlayer_Stop!");
            OH_AVPlayer_Stop(self.ohos_av_player);
        }
    }

    pub fn prepare(&mut self) {
        unsafe {
            debug!("OH_AVPlayer Prepare Called!");
            OH_AVPlayer_Prepare(self.ohos_av_player);
        }
    }

    // For AVPlayer only call SetSource after SetInputSize to avoid being recognized as live stream.
    pub fn set_input_size(&mut self, size: u64) {
        if let Some(inner_source) = &mut self.media_data_source {
            debug!("Setting up data source size: {}", size);
            inner_source.set_input_size(size as usize);
            // Only set once when first time initialized
            if !self.has_set_source_size {
                debug!("Setup data Source");
                self.setup_data_source();
                self.has_set_source_size = true;
            }
        }
    }

    pub fn connect_info_event_callback<F>(&mut self, f: F)
    where
        F: Fn(AVPlayerOnInfoType, *mut OH_AVFormat) + Send + 'static,
    {
        debug!("Trying to connect info event callback");
        extern "C" fn on_info_event(
            _player: *mut OH_AVPlayer,
            into_type: AVPlayerOnInfoType,
            info_body: *mut OH_AVFormat,
            user_data: *mut c_void,
        ) {
            assert!(
                !user_data.is_null(),
                "on_info_event: user_data must not be null"
            );
            let f = unsafe {
                &*(user_data as *const Box<dyn Fn(AVPlayerOnInfoType, *mut OH_AVFormat)>)
            };
            f(into_type, info_body);
        }

        let f: Box<dyn Fn(AVPlayerOnInfoType, *mut OH_AVFormat)> = Box::new(f);
        let f: Box<Box<dyn Fn(AVPlayerOnInfoType, *mut OH_AVFormat)>> = Box::new(f);
        let raw_ptr_f = unsafe {
            let raw_ptr_f = Box::into_raw(f);
            let ret = OH_AVPlayer_SetOnInfoCallback(
                self.ohos_av_player,
                Some(on_info_event),
                raw_ptr_f as *mut c_void,
            );
            debug!("OH AVPlayer Set INFO Callback: {:?}", ret);
            raw_ptr_f
        };
        self.event_info_callback_closure = Some(raw_ptr_f);
    }

    /// External Initialization step.
    pub fn setup_window_buffer_listener<F: Fn() + Send + 'static>(&mut self, f: F) {
        let f: Box<dyn Fn()> = Box::new(f);
        let f: Box<Box<dyn Fn()>> = Box::new(f);
        (
            self.native_image,
            self.frame_available_callback_closure,
            self.native_window,
        ) = unsafe {
            let native_image = OH_ConsumerSurface_Create();

            debug!("Native image created :{:p}", native_image);
            let ret = OH_ConsumerSurface_SetDefaultUsage(
                native_image,
                OH_NativeBuffer_Usage::NATIVEBUFFER_USAGE_CPU_READ.0 as u64,
            );
            debug!("Set consumer surface default usage: {}", ret);

            extern "C" fn frame_available_cb(context: *mut c_void) {
                assert!(
                    !context.is_null(),
                    "frame_available_cb: context must not be null"
                );
                let f = unsafe { &*(context as *mut Box<dyn Fn()>) };
                f();
            }

            let raw_ptr_f = Box::into_raw(f);
            let listener = OH_OnFrameAvailableListener {
                context: raw_ptr_f as *mut c_void,
                onFrameAvailable: Some(frame_available_cb),
            };
            let res = OH_NativeImage_SetOnFrameAvailableListener(native_image, listener);
            debug!("Native Image Set On Frame Available Listener done: {}", res);

            let native_window = OH_NativeImage_AcquireNativeWindow(native_image);

            debug!(
                "Native window acquired from native window {:p}",
                native_window
            );
            (Some(native_image), Some(raw_ptr_f), Some(native_window))
        };
        self.initialize_check_state_action();
    }

    /// Should pair with release_buffer.
    pub fn acquire_buffer(&self) -> Option<FrameInfo> {
        let native_image = self.native_image?;
        let mut native_window_buffer = std::ptr::null_mut();
        let mut fence_fd = 0;
        let ret = unsafe {
            OH_NativeImage_AcquireNativeWindowBuffer(
                native_image,
                &mut native_window_buffer,
                &mut fence_fd,
            )
        };
        if ret != 0 || native_window_buffer.is_null() {
            warn!("Failed to acquire native window buffer: ret={}", ret);
            return None;
        }
        debug!("Fence fd: {}", fence_fd);
        if fence_fd != 0 && fence_fd != -1 {
            let mut pollfds = pollfd {
                fd: fence_fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = unsafe { libc::poll(&mut pollfds, 1, 3000) };
            if ret <= 0 {
                warn!("Pulling timeout or failed");
                return None;
            }
        }
        debug!("Taking object refernce!");
        let ret =
            unsafe { OH_NativeWindow_NativeObjectReference(native_window_buffer as *mut c_void) };
        if ret != 0 {
            warn!("Native Window Buffer Reference Failed!");
        }

        let frame_info = unsafe {
            let buffer_handle = OH_NativeWindow_GetBufferHandleFromNative(native_window_buffer);
            FrameInfo {
                fd: (*buffer_handle).fd,
                width: (*buffer_handle).width,
                height: (*buffer_handle).height,
                stride: (*buffer_handle).stride,
                size: (*buffer_handle).size,
                format: (*buffer_handle).format,
                vir_addr: (*buffer_handle).virAddr as *mut u8,
                native_window_buffer,
                fence_fd,
            }
        };
        let ret =
            unsafe { OH_NativeWindow_NativeObjectUnreference(native_window_buffer as *mut c_void) };
        if ret != 0 {
            warn!("Native Window Buffer Unreference failed!");
        }
        // FIXME(ray): Potential memory copying.
        Some(frame_info)
    }

    /// Should pair with acquire_buffer.
    pub fn release_buffer(&self, frame_info: FrameInfo) {
        let native_image = self.native_image.expect("native image should not be empty");
        unsafe {
            let ret = OH_NativeImage_ReleaseNativeWindowBuffer(
                native_image,
                frame_info.native_window_buffer,
                -1,
            );
            debug!("Release native window buffer ret: {}", ret);
        }
    }
}

impl Drop for OhosPlayer {
    fn drop(&mut self) {
        unsafe {
            if let Some(closure) = self.frame_available_callback_closure {
                let box_closure = Box::from_raw(closure);
                drop(box_closure);
            }
            if let Some(closure) = self.event_info_callback_closure {
                let box_closure = Box::from_raw(closure);
                drop(box_closure);
            }
            debug!("Releasing AVPlayer because drop is called!");
            OH_AVPlayer_Release(self.ohos_av_player);
            if let Some(mut native_image) = self.native_image {
                OH_NativeImage_Destroy(&mut native_image);
            }
            if let Some(native_window) = self.native_window {
                OH_NativeWindow_DestroyNativeWindow(native_window);
            }
        }
    }
}

unsafe impl Send for OhosPlayer {}
unsafe impl Sync for OhosPlayer {}
