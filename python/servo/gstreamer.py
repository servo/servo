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
    ("gstapp", "gst-plugins-base"),
    ("gstaudio", "gst-plugins-base"),
    ("gstbase", "gstreamer"),
    ("gstcodecparsers", "gst-plugins-bad"),
    ("gstcontroller", "gstreamer"),
    ("gstfft", "gst-plugins-base"),
    ("gstgl", "gst-plugins-base"),
    ("gstnet", "gstreamer"),
    ("gstpbutils", "gst-plugins-base"),
    ("gstplayer", "gst-plugins-bad"),
    ("gstreamer", "gstreamer"),
    ("gstriff", "gst-plugins-base"),
    ("gstrtp", "gst-plugins-base"),
    ("gstrtsp", "gst-plugins-base"),
    ("gstsctp", "gst-plugins-bad"),
    ("gstsdp", "gst-plugins-base"),
    ("gsttag", "gst-plugins-base"),
    ("gstvideo", "gst-plugins-base"),
    ("gstwebrtc", "gst-plugins-bad"),
]

NON_UWP_DYLIBS = [
    "gstnet",
    "gstsctp",
]

GSTREAMER_PLUGINS = [
    ("gstapp", "gst-plugins-base"),
    ("gstaudiobuffersplit", "gst-plugins-bad"),
    ("gstaudioconvert", "gst-plugins-base"),
    ("gstaudiofx", "gst-plugins-good"),
    ("gstaudioparsers", "gst-plugins-good"),
    ("gstaudioresample", "gst-plugins-base"),
    ("gstautodetect", "gst-plugins-good"),
    ("gstcoreelements", "gstreamer"),
    ("gstdeinterlace", "gst-plugins-good"),
    ("gstdtls", "gst-plugins-bad"),
    ("gstgio", "gst-plugins-base"),
    ("gstid3tag", "gst-plugins-bad"),
    ("gstid3demux", "gst-plugins-good"),
    ("gstinterleave", "gst-plugins-good"),
    ("gstisomp4", "gst-plugins-good"),
    ("gstlibav", "gst-libav"),
    ("gstmatroska", "gst-plugins-good"),
    ("gstogg", "gst-plugins-base"),
    ("gstopengl", "gst-plugins-base"),
    ("gstopus", "gst-plugins-base"),
    ("gstplayback", "gst-plugins-base"),
    ("gstproxy", "gst-plugins-bad"),
    ("gstrtp", "gst-plugins-good"),
    ("gstrtpmanager", "gst-plugins-good"),
    ("gsttheora", "gst-plugins-base"),
    ("gsttypefindfunctions", "gst-plugins-base"),
    ("gstvideoconvert", "gst-plugins-base"),
    ("gstvideofilter", "gst-plugins-good"),
    ("gstvideoparsersbad", "gst-plugins-bad"),
    ("gstvideoscale", "gst-plugins-base"),
    ("gstvorbis", "gst-plugins-base"),
    ("gstvolume", "gst-plugins-base"),
    ("gstvpx", "gst-plugins-good"),
    ("gstwebrtc", "gst-plugins-bad"),
]

WINDOWS_PLUGINS = [
    "gstnice",
    "gstwasapi",
]

MACOS_PLUGINS = [
    ("gstapplemedia", "gst-plugins-bad"),
    ("gstosxaudio", "gst-plugins-good"),
    ("gstosxvideo", "gst-plugins-good"),
]

NON_UWP_PLUGINS = [
    "gstdtls",
    "gstmatroska",
    "gstnice",
    "gstogg",
    "gstopengl",
    "gstopus",
    "gstrtp",
    "gstrtpmanager",
    "gsttheora",
    "gstvorbis",
    "gstvpx",
    "gstwebrtc",
]


def windows_dlls(uwp):
    dlls = [x for x, _ in GSTREAMER_DYLIBS]
    if uwp:
        dlls = filter(lambda x: x not in NON_UWP_DYLIBS, dlls)
    return [x + "-1.0-0.dll" for x in dlls]


def windows_plugins(uwp):
    dlls = [x for x, _ in GSTREAMER_PLUGINS] + WINDOWS_PLUGINS
    if uwp:
        dlls = filter(lambda x: x not in NON_UWP_PLUGINS, dlls)
    return [x + ".dll" for x in dlls]


def macos_libnice():
    return os.path.join('/', 'usr', 'local', 'opt', 'libnice', 'lib')


def macos_dylibs():
    return [
        os.path.join(
            "/usr/local/opt",
            path,
            "lib",
            "lib" + name + "-1.0.0.dylib"
        ) for name, path in GSTREAMER_DYLIBS
    ] + [
        os.path.join(macos_libnice(), "libnice.dylib"),
        os.path.join(macos_libnice(), "libnice.10.dylib"),
        os.path.join(
            "/usr/local/opt",
            "gstreamer",
            "lib",
            "gstreamer-1.0",
            "libgstcoreelements.dylib",
        ),
    ]


def macos_plugins():
    # As of 1.18.1, gstcoreelements is one of two plugins with a .dylib
    # that's under /lib/gstreamer-1.0, so it needs to be handled separately.
    plugin_exceptions = ["gstcoreelements"]
    plugins = filter(lambda x: x[0] not in plugin_exceptions,
                     GSTREAMER_PLUGINS + MACOS_PLUGINS)
    return [
        os.path.join(
            "/usr/local/opt",
            path,
            "lib",
            "gstreamer-1.0",
            "lib" + name + ".so"
        ) for name, path in plugins
    ] + [
        os.path.join(macos_libnice(), "gstreamer-1.0", "libgstnice.so"),
    ]


def write_plugin_list(target):
    plugins = []
    if "apple-" in target:
        plugins = [os.path.basename(x) for x in macos_plugins()]
    elif '-windows-' in target:
        plugins = windows_plugins('-uwp-' in target)
    print('''/* This is a generated file. Do not modify. */

pub(crate) static GSTREAMER_PLUGINS: &[&'static str] = &[
%s
];
''' % ',\n'.join(map(lambda x: '"' + x + '"', plugins)))


if __name__ == "__main__":
    write_plugin_list(sys.argv[1])
