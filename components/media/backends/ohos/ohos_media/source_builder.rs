/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#[cfg(not(sdk_api_21))]
use crate::ohos_media::dummy_source::MediaSourceWrapper;
#[cfg(sdk_api_21)]
use crate::ohos_media::source::MediaSourceWrapper;

type SeekDataClosure = Box<dyn Fn(u64) -> bool + Send + Sync>;

pub struct MediaSourceBuilder {
    pub enough_data: Option<Box<dyn Fn() + Send + Sync>>,
    pub seek_data: Option<SeekDataClosure>,
}

impl MediaSourceBuilder {
    pub fn set_enough_data<F: Fn() + Send + Sync + Clone + 'static>(mut self, callback: F) -> Self {
        self.enough_data = Some(Box::new(callback));
        self
    }

    pub fn set_seek_data<F: Fn(u64) -> bool + Send + Sync + Clone + 'static>(
        mut self,
        callback: F,
    ) -> Self {
        self.seek_data = Some(Box::new(callback));
        self
    }

    pub fn build(self) -> MediaSourceWrapper {
        #[cfg(not(sdk_api_21))]
        {
            MediaSourceWrapper::new()
        }
        #[cfg(sdk_api_21)]
        {
            MediaSourceWrapper::new(self)
        }
    }
}
