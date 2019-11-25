/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glib::subclass::types::ObjectSubclass;
use gstreamer::gst_plugin_define;
use servosrc::ServoSrc;

mod logging;
mod resources;
mod servosrc;

gst_plugin_define!(
    servoplugin,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION"), "-", env!("COMMIT_ID")),
    "MPL",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY"),
    env!("BUILD_REL_DATE")
);

fn plugin_init(plugin: &gstreamer::Plugin) -> Result<(), glib::BoolError> {
    gstreamer::gst_debug!(logging::CATEGORY, "Initializing logging");
    log::set_logger(&logging::LOGGER).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Debug);

    log::debug!("Initializing resources");
    resources::init();

    log::debug!("Registering plugin");
    gstreamer::Element::register(
        Some(plugin),
        "servosrc",
        gstreamer::Rank::None,
        ServoSrc::get_type(),
    )
}
