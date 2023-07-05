# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import sys

GSTREAMER_DYLIBS = [
    # gstreamer
    "gstbase",
    "gstcontroller",
    "gstnet",
    "gstreamer",
    # gst-plugins-base
    "gstapp",
    "gstaudio",
    "gstfft",
    "gstgl",
    "gstpbutils",
    "gstriff",
    "gstrtp",
    "gstrtsp",
    "gstsctp",
    "gstsdp",
    "gsttag",
    "gstvideo",
    # gst-plugins-bad
    "gstcodecparsers",
    "gstplayer",
    "gstwebrtc",
]


GSTREAMER_PLUGINS = [
    # gstreamer
    "gstcoreelements",
    "gstnice",
    # gst-plugins-base
    "gstapp",
    "gstaudioconvert",
    "gstaudioresample",
    "gstgio",
    "gstogg",
    "gstopengl",
    "gstopus",
    "gstplayback",
    "gsttheora",
    "gsttypefindfunctions",
    "gstvolume",
    "gstvorbis",
    # gst-plugins-good
    "gstaudiofx",
    "gstaudioparsers",
    "gstautodetect",
    "gstdeinterlace",
    "gstid3demux",
    "gstinterleave",
    "gstisomp4",
    "gstmatroska",
    "gstrtp",
    "gstrtpmanager",
    "gstvideofilter",
    "gstvpx",
    # gst-plugins-bad
    "gstaudiobuffersplit",
    "gstdtls",
    "gstid3tag",
    "gstproxy",
    "gstvideoparsersbad",
    "gstwebrtc",
    # gst-libav
    "gstlibav",
]


def windows_dlls():
    libs = list(GSTREAMER_DYLIBS)
    return [f"{lib}-1.0-0.dll" for lib in libs]


def windows_plugins():
    # FIXME: We should support newer gstreamer versions here that replace
    # gstvideoconvert and gstvideoscale with gstvideoconvertscale.
    libs = [
        *GSTREAMER_PLUGINS,
        "gstvideoconvert",
        "gstvideoscale",
        "gstwasapi"
    ]
    return [f"{lib}.dll" for lib in libs]


def macos_gst_root():
    return os.path.join(
        "/", "Library", "Frameworks", "GStreamer.framework", "Versions", "1.0")


def macos_plugins():
    plugins = [
        *GSTREAMER_PLUGINS,
        # gst-plugins-good
        "gstosxaudio",
        "gstosxvideo",
        # gst-plugins-bad
        "gstapplemedia",
    ]

    def plugin_path(plugin):
        return os.path.join(macos_gst_root(), 'lib', 'gstreamer-1.0', f"lib{plugin}.dylib")

    # These plugins depend on the particular version of GStreamer that is installed
    # on the system that is building servo.
    conditional_plugins = [
        # gst-plugins-base
        plugin_path("gstvideoconvert"),
        plugin_path("gstvideoscale"),
        plugin_path("gstvideoconvertscale")
    ]
    conditional_plugins = list(filter(lambda path: os.path.exists(path),
                                      conditional_plugins))
    return [plugin_path(plugin) for plugin in plugins] + conditional_plugins


def write_plugin_list(target):
    plugins = []
    if "apple-" in target:
        plugins = [os.path.basename(x) for x in macos_plugins()]
    elif '-windows-' in target:
        plugins = windows_plugins()
    print('''/* This is a generated file. Do not modify. */

pub(crate) static GSTREAMER_PLUGINS: &[&'static str] = &[
%s
];
''' % ',\n'.join(map(lambda x: '"' + x + '"', plugins)))


if __name__ == "__main__":
    write_plugin_list(sys.argv[1])
