use servo_media::player::frame::{Frame, FrameRenderer};
use std::mem;
use std::sync::{Arc, Mutex};
use webrender_api::{
    ImageData, ImageDescriptor, ImageFormat, ImageKey, RenderApi, RenderApiSender, Transaction,
};

#[derive(Clone)]
pub struct MediaFrameRenderer {
    inner: Arc<Mutex<MediaFrameRendererInner>>,
}

impl MediaFrameRenderer {
    pub fn new(render_api_sender: RenderApiSender) -> MediaFrameRenderer {
        MediaFrameRenderer {
            inner: Arc::new(Mutex::new(MediaFrameRendererInner {
                api: render_api_sender.create_api(),
                current_frame: None,
                old_frame: None,
                very_old_frame: None,
            })),
        }
    }
}

impl FrameRenderer for MediaFrameRenderer {
    fn render(&self, frame: Frame) {
        self.inner.lock().unwrap().render(frame);
    }
}

struct MediaFrameRendererInner {
    api: RenderApi,
    current_frame: Option<(ImageKey, i32, i32)>,
    old_frame: Option<ImageKey>,
    very_old_frame: Option<ImageKey>,
}

impl MediaFrameRendererInner {
    fn render(&mut self, frame: Frame) {
        let descriptor = ImageDescriptor::new(
            frame.get_width() as u32,
            frame.get_height() as u32,
            ImageFormat::BGRA8,
            false,
            false,
        );

        let mut txn = Transaction::new();

        //let image_data = ImageData::new_shared(frame.get_data().clone());
        let image_data = ImageData::Raw(frame.get_data().clone());

        if let Some(old_image_key) = mem::replace(&mut self.very_old_frame, self.old_frame.take()) {
            txn.delete_image(old_image_key);
        }

        match self.current_frame {
            Some((ref image_key, ref mut width, ref mut height))
                if *width == frame.get_width() && *height == frame.get_height() =>
            {
                txn.update_image(*image_key, descriptor, image_data, None);

                if let Some(old_image_key) = self.old_frame.take() {
                    txn.delete_image(old_image_key);
                }
            }
            Some((ref mut image_key, ref mut width, ref mut height)) => {
                self.old_frame = Some(*image_key);

                let new_image_key = self.api.generate_image_key();
                txn.add_image(new_image_key, descriptor, image_data, None);
                *image_key = new_image_key;
                *width = frame.get_width();
                *height = frame.get_height();
            }
            None => {
                let image_key = self.api.generate_image_key();
                txn.add_image(image_key, descriptor, image_data, None);
                self.current_frame = Some((image_key, frame.get_width(), frame.get_height()));
            }
        }

        self.api.update_resources(txn.resource_updates);
    }
}
