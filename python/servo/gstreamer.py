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
    "gstsctp",
]

GSTREAMER_PLUGINS = [
    ("gstapp", "gst-plugins-base"),
    ("gstaudioconvert", "gst-plugins-base"),
    ("gstaudiofx", "gst-plugins-good"),
    ("gstaudioparsers", "gst-plugins-good"),
    ("gstaudioresample", "gst-plugins-base"),
    ("gstautodetect", "gst-plugins-good"),
    ("gstcoreelements", "gstreamer"),
    ("gstdeinterlace", "gst-plugins-good"),
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
    ("gstnice", "gst-plugins-base"),
    ("gstwasapi", "gst-plugins-base"),
]

MACOS_PLUGINS = [
    ("gstapplemedia", "gst-plugins-bad"),
]

NON_UWP_PLUGINS = [
    "gstmatroska",
    "gstnice",
    "gstogg",
    "gstopengl",
    "gstopus",
    "gstrtp",
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
    dlls = [x for x, _ in GSTREAMER_PLUGINS] + [x for x, _ in WINDOWS_PLUGINS]
    if uwp:
        dlls = filter(lambda x: x not in NON_UWP_PLUGINS, dlls)
    return [x + ".dll" for x in dlls]


def macos_dylibs():
    return [
        os.path.join(
            "/usr/local/opt",
            path,
            "lib",
            "lib" + name + "-1.0.0.dylib"
        ) for name, path in GSTREAMER_DYLIBS
    ]


def macos_plugins():
    return [
        os.path.join(
            "/usr/local/opt",
            path,
            "lib",
            "gstreamer-1.0",
            "lib" + name + ".so"
        ) for name, path in GSTREAMER_PLUGINS
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
