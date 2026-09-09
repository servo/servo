/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::{Arc, Mutex, RwLock, Weak};

use euclid::default::Size2D;
use layout_api::MediaFrame;
use media::{GLPlayerMsg, GLPlayerMsgForward, WindowGLContext};
use net_traits::image_cache::{EncodedImage, Image};
use paint_api::{CrossProcessPaintApi, ImageUpdate, SerializableImageData};
use servo_base::generic_channel::{GenericCallback, GenericSharedMemory};
use servo_base::id::WebViewId;
use servo_media::player::video::{VideoFrame, VideoFrameRenderer};
use webrender_api::{
    ExternalImageData, ExternalImageId, ExternalImageType, ImageBufferKind, ImageDescriptor,
    ImageDescriptorFlags, ImageFormat, ImageKey,
};

use crate::tasks::task_source::SendableTaskSource;

#[derive(MallocSizeOf, PartialEq)]
enum FrameStatus {
    Locked,
    Unlocked,
}

#[derive(MallocSizeOf)]
pub(crate) struct FrameHolder(FrameStatus, VideoFrame);

impl FrameHolder {
    fn new(frame: VideoFrame) -> FrameHolder {
        FrameHolder(FrameStatus::Unlocked, frame)
    }

    fn lock(&mut self) {
        if self.0 == FrameStatus::Unlocked {
            self.0 = FrameStatus::Locked;
        };
    }

    fn unlock(&mut self) {
        if self.0 == FrameStatus::Locked {
            self.0 = FrameStatus::Unlocked;
        };
    }

    fn set(&mut self, new_frame: VideoFrame) {
        if self.0 == FrameStatus::Unlocked {
            self.1 = new_frame
        };
    }

    fn get(&self) -> (u32, Size2D<i32>, usize) {
        if self.0 == FrameStatus::Locked {
            (
                self.1.get_texture_id(),
                Size2D::new(self.1.get_width(), self.1.get_height()),
                0,
            )
        } else {
            unreachable!();
        }
    }

    pub(crate) fn get_frame(&self) -> VideoFrame {
        self.1.clone()
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct MediaFrameRenderer {
    webview_id: WebViewId,
    player_id: Option<usize>,
    #[conditional_malloc_size_of]
    glplayer_id: Arc<RwLock<Option<u64>>>,
    paint_api: CrossProcessPaintApi,
    player_context: WindowGLContext,
    current_frame: Option<MediaFrame>,
    old_frame: Option<ImageKey>,
    very_old_frame: Option<ImageKey>,
    current_frame_holder: Option<FrameHolder>,
    /// <https://html.spec.whatwg.org/multipage/#poster-frame>
    poster_frame: Option<MediaFrame>,
    #[conditional_malloc_size_of]
    static_poster: Option<Arc<EncodedImage>>,
}

impl MediaFrameRenderer {
    pub(crate) fn new(
        webview_id: WebViewId,
        paint_api: CrossProcessPaintApi,
        player_context: WindowGLContext,
    ) -> Self {
        Self {
            webview_id,
            player_id: None,
            glplayer_id: Arc::new(RwLock::new(None)),
            paint_api,
            player_context,
            current_frame: None,
            old_frame: None,
            very_old_frame: None,
            current_frame_holder: None,
            poster_frame: None,
            static_poster: None,
        }
    }

    pub(crate) fn current_frame(&self) -> Option<MediaFrame> {
        self.current_frame
    }

    pub(crate) fn poster_frame(&self) -> Option<MediaFrame> {
        self.poster_frame
    }

    pub(crate) fn static_poster(&self) -> Option<Arc<EncodedImage>> {
        self.static_poster.clone()
    }

    pub(crate) fn current_frame_holder(&self) -> Option<&FrameHolder> {
        self.current_frame_holder.as_ref()
    }

    pub(crate) fn setup(
        &mut self,
        player_id: usize,
        task_source: SendableTaskSource,
        weak_video_renderer: Weak<Mutex<MediaFrameRenderer>>,
    ) {
        self.player_id = Some(player_id);
        let glplayer_id = self.glplayer_id.clone();
        let callback = GenericCallback::new(move |message| {
            let message = message.unwrap();
            let weak_video_renderer = weak_video_renderer.clone();

            let glplayer_id = glplayer_id.clone();
            task_source.queue(task!(handle_glplayer_message: move || {
                trace!("GLPlayer message {:?}", message);

                let Some(video_renderer) = weak_video_renderer.upgrade() else {
                    return;
                };

                match message {
                    GLPlayerMsgForward::Lock(sender) => {
                        if let Some(holder) = video_renderer
                            .lock()
                            .unwrap()
                            .current_frame_holder
                            .as_mut() {
                                holder.lock();
                                sender.send(holder.get()).unwrap();
                            };
                    },
                    GLPlayerMsgForward::Unlock() => {
                        if let Some(holder) = video_renderer
                            .lock()
                            .unwrap()
                            .current_frame_holder
                            .as_mut() { holder.unlock() }
                    },
                    GLPlayerMsgForward::PlayerId(id) => {
                        let mut glplayer_id = glplayer_id.write().unwrap();
                        if let Some(already_set_id) = *glplayer_id {
                            error!("Player id already set to {already_set_id} will be replaced with {id}");
                        }
                        *glplayer_id = Some(id);
                    },
                }
            }));
        })
        .expect("Could not create callback");

        if let Some(glplayer_thread_sender) = &self.player_context.glplayer_thread_sender {
            glplayer_thread_sender
                .send(GLPlayerMsg::RegisterPlayer(callback))
                .unwrap();
        }
    }

    pub(crate) fn reset(&mut self) {
        self.player_id = None;

        if let Some(glplayer_id) = self.glplayer_id.write().unwrap().take() {
            self.player_context
                .send(GLPlayerMsg::UnregisterPlayer(glplayer_id));
        }

        self.current_frame_holder = None;

        let mut updates = smallvec::smallvec![];

        if let Some(current_frame) = self.current_frame.take() {
            updates.push(ImageUpdate::DeleteImage(current_frame.image_key));
        }

        if let Some(old_image_key) = self.old_frame.take() {
            updates.push(ImageUpdate::DeleteImage(old_image_key));
        }

        if let Some(very_old_image_key) = self.very_old_frame.take() {
            updates.push(ImageUpdate::DeleteImage(very_old_image_key));
        }

        if !updates.is_empty() {
            self.paint_api
                .update_images(self.webview_id.into(), updates);
        }
    }

    pub(crate) fn set_poster_frame(&mut self, image: Option<Image>) {
        self.static_poster = match &image {
            Some(Image::Encoded(source)) => Some(source.clone()),
            _ => None,
        };
        self.poster_frame = image.and_then(|image| {
            let Image::Raster(image) = image else {
                return None;
            };
            image.id.map(|image_key| MediaFrame {
                image_key,
                width: image.metadata.width as i32,
                height: image.metadata.height as i32,
            })
        });
    }
}

impl Drop for MediaFrameRenderer {
    fn drop(&mut self) {
        self.reset();
    }
}

impl VideoFrameRenderer for MediaFrameRenderer {
    fn render(&mut self, frame: VideoFrame) {
        if self.player_id.is_none() ||
            (frame.is_gl_texture() && self.glplayer_id.read().unwrap().is_none())
        {
            return;
        }

        let mut updates = smallvec::smallvec![];

        if let Some(old_image_key) =
            std::mem::replace(&mut self.very_old_frame, self.old_frame.take())
        {
            updates.push(ImageUpdate::DeleteImage(old_image_key));
        }

        let descriptor = ImageDescriptor::new(
            frame.get_width(),
            frame.get_height(),
            ImageFormat::BGRA8,
            ImageDescriptorFlags::empty(),
        );

        match &mut self.current_frame {
            Some(current_frame)
                if current_frame.width == frame.get_width() &&
                    current_frame.height == frame.get_height() =>
            {
                if !frame.is_gl_texture() {
                    updates.push(ImageUpdate::UpdateImage(
                        current_frame.image_key,
                        descriptor,
                        SerializableImageData::Raw(GenericSharedMemory::from_arc_vec(
                            frame.get_data(),
                        )),
                        None,
                    ));
                }

                self.current_frame_holder
                    .get_or_insert_with(|| FrameHolder::new(frame.clone()))
                    .set(frame);

                if let Some(old_image_key) = self.old_frame.take() {
                    updates.push(ImageUpdate::DeleteImage(old_image_key));
                }
            },
            Some(current_frame) => {
                self.old_frame = Some(current_frame.image_key);

                let Some(new_image_key) =
                    self.paint_api.generate_image_key_blocking(self.webview_id)
                else {
                    return;
                };

                /* update current_frame */
                current_frame.image_key = new_image_key;
                current_frame.width = frame.get_width();
                current_frame.height = frame.get_height();

                // FIXME: This code is duplicated below this branch
                let image_data = self
                    .glplayer_id
                    .read()
                    .unwrap()
                    .filter(|_| frame.is_gl_texture())
                    .map(|glplayer_id| {
                        let texture_target = if frame.is_external_oes() {
                            ImageBufferKind::TextureExternal
                        } else {
                            ImageBufferKind::Texture2D
                        };

                        SerializableImageData::External(ExternalImageData {
                            id: ExternalImageId(glplayer_id),
                            channel_index: 0,
                            image_type: ExternalImageType::TextureHandle(texture_target),
                            normalized_uvs: false,
                        })
                    })
                    .unwrap_or_else(|| {
                        SerializableImageData::Raw(GenericSharedMemory::from_arc_vec(
                            frame.get_data(),
                        ))
                    });

                self.current_frame_holder
                    .get_or_insert_with(|| FrameHolder::new(frame.clone()))
                    .set(frame);

                updates.push(ImageUpdate::AddImage(
                    new_image_key,
                    descriptor,
                    image_data,
                    false,
                ));
            },
            None => {
                let Some(image_key) = self.paint_api.generate_image_key_blocking(self.webview_id)
                else {
                    return;
                };

                self.current_frame = Some(MediaFrame {
                    image_key,
                    width: frame.get_width(),
                    height: frame.get_height(),
                });

                let image_data = self
                    .glplayer_id
                    .read()
                    .unwrap()
                    .filter(|_| frame.is_gl_texture())
                    .map(|glplayer_id| {
                        let texture_target = if frame.is_external_oes() {
                            ImageBufferKind::TextureExternal
                        } else {
                            ImageBufferKind::Texture2D
                        };

                        SerializableImageData::External(ExternalImageData {
                            id: ExternalImageId(glplayer_id),
                            channel_index: 0,
                            image_type: ExternalImageType::TextureHandle(texture_target),
                            normalized_uvs: false,
                        })
                    })
                    .unwrap_or_else(|| {
                        SerializableImageData::Raw(GenericSharedMemory::from_arc_vec(
                            frame.get_data(),
                        ))
                    });

                self.current_frame_holder = Some(FrameHolder::new(frame));

                updates.push(ImageUpdate::AddImage(
                    image_key, descriptor, image_data, false,
                ));
            },
        }
        self.paint_api
            .update_images(self.webview_id.into(), updates);
    }
}
