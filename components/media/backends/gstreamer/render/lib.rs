//! `Render` is a trait to be used by GStreamer's backend player
//!
//! The purpose of this trait is to provide different accelerated
//! video renders.
//!
//! By default, the player will use a rendering mechanism based on
//! mapping the raw video into CPU memory, but it might be other
//! rendering mechanism. The main target for this trait are
//! OpenGL-based render mechanisms.
//!
//! Each platform (Unix, MacOS, Windows) might offer an implementation
//! of this trait, so the player could setup a proper GStreamer
//! pipeline, and handle the produced buffers.
//!

pub trait Render {
    /// Returns `True` if the render implementation uses any version
    /// or flavor of OpenGL
    fn is_gl(&self) -> bool;

    /// Returns the Player's `Frame` to be consumed by the API user.
    ///
    /// The implementation of this method will map the `sample`'s
    /// buffer to the rendering appropriate structure. In the case of
    /// OpenGL-based renders, the `Frame`, instead of the raw data,
    /// will transfer the texture ID.
    ///
    /// # Arguments
    ///
    /// * `sample` -  the GStreamer sample with the buffer to map
    fn build_frame(&self, sample: gstreamer::Sample) -> Option<sm_player::video::VideoFrame>;

    /// Sets the proper *video-sink* to GStreamer's `pipeline`, this
    /// video sink is simply a decorator of the passed `appsink`.
    ///
    /// # Arguments
    ///
    /// * `appsink` - the appsink GStreamer element to decorate
    /// * `pipeline` - the GStreamer pipeline to set the video sink
    fn build_video_sink(
        &self,
        appsink: &gstreamer::Element,
        pipeline: &gstreamer::Element,
    ) -> Result<(), sm_player::PlayerError>;
}
