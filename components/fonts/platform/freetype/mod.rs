/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod font;
mod freetype_face;

#[cfg(all(target_os = "linux", not(target_env = "ohos"), not(ohos_mock)))]
pub mod font_list;

#[cfg(target_os = "android")]
mod android {
    pub mod font_list;
}
#[cfg(target_os = "android")]
pub use self::android::font_list;

#[cfg(any(target_env = "ohos", ohos_mock))]
mod ohos {
    pub mod font_list;
}
#[cfg(any(target_env = "ohos", ohos_mock))]
pub use self::ohos::font_list;

mod library_handle;
