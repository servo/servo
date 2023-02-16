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
import platform

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


def windows_dlls(uwp):
    libs = list(GSTREAMER_DYLIBS)
    NON_UWP_DYLIBS = [
        "gstnet",
        "gstsctp",
    ]
    if uwp:
        libs = filter(lambda x: x not in NON_UWP_DYLIBS, libs)
    return [f"{lib}-1.0-0.dll" for lib in libs]


def windows_plugins(uwp):
    # FIXME: We should support newer gstreamer versions here that replace
    # gstvideoconvert and gstvideoscale with gstvideoconvertscale.
    libs = [
        *GSTREAMER_PLUGINS,
        "gstvideoconvert",
        "gstvideoscale",
        "gstwasapi"
    ]
    NON_UWP_PLUGINS = [
        "gstnice",
        # gst-plugins-base
        "gstogg",
        "gstopengl",
        "gstopus",
        "gstrtp",
        "gsttheora",
        "gstvorbis",
        # gst-plugins-good
        "gstmatroska",
        "gstrtpmanager",
        "gstvpx",
        # gst-plugins-bad
        "gstdtls",
        "gstwebrtc",
    ]
    if uwp:
        libs = filter(lambda x: x not in NON_UWP_PLUGINS, libs)
    return [f"{lib}.dll" for lib in libs]


def macos_lib_dir():
    # homebrew use /opt/homebrew on macos ARM, use /usr/local on Intel
    if platform.machine() == 'arm64':
        return os.path.join('/', 'opt', 'homebrew', 'lib')
    return os.path.join('/', 'usr', 'local', 'lib')


def macos_dylibs():
    dylibs = [
        *[f"lib{lib}-1.0.0.dylib" for lib in GSTREAMER_DYLIBS],
        "libnice.dylib",
        "libnice.10.dylib",
    ]
    return [os.path.join(macos_lib_dir(), lib) for lib in dylibs]


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
        return os.path.join(macos_lib_dir(), 'gstreamer-1.0', f"lib{plugin}.dylib")

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
        plugins = windows_plugins('-uwp-' in target)
    print('''/* This is a generated file. Do not modify. */

pub(crate) static GSTREAMER_PLUGINS: &[&'static str] = &[
%s
];
''' % ',\n'.join(map(lambda x: '"' + x + '"', plugins)))


if __name__ == "__main__":
    write_plugin_list(sys.argv[1])
