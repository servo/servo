/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_media::player::context::{GlApi, GlContext, NativeDisplay, PlayerGLContext};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WindowGLContext {
    /// Application's GL Context
    pub gl_context: GlContext,
    /// Application's GL Api
    pub gl_api: GlApi,
    /// Application's native display
    pub native_display: NativeDisplay,
}

impl PlayerGLContext for WindowGLContext {
    fn get_gl_context(&self) -> GlContext {
        self.gl_context.clone()
    }

    fn get_native_display(&self) -> NativeDisplay {
        self.native_display.clone()
    }

    fn get_gl_api(&self) -> GlApi {
        self.gl_api.clone()
    }
}
