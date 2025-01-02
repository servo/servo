from typing import Any

from Cocoa import NSURL
from ColorSync import (
    CGDisplayCreateUUIDFromDisplayID,
    ColorSyncDeviceSetCustomProfiles,
    kColorSyncDeviceDefaultProfileID,
    kColorSyncDisplayDeviceClass,
)
from Quartz import (
    CGGetOnlineDisplayList,
    kCGErrorSuccess,
)


def set_all_displays(profile_url: NSURL) -> bool:
    max_displays = 10

    (err, display_ids, display_count) = CGGetOnlineDisplayList(max_displays, None, None)
    if err != kCGErrorSuccess:
        raise ValueError(err)

    display_uuids = [CGDisplayCreateUUIDFromDisplayID(d) for d in display_ids]

    for display_uuid in display_uuids:
        profile_info = {kColorSyncDeviceDefaultProfileID: profile_url}

        success = ColorSyncDeviceSetCustomProfiles(
            kColorSyncDisplayDeviceClass,
            display_uuid,
            profile_info,
        )
        if not success:
            raise Exception(f"failed to set profile on {display_uuid}")

    return True


def run(venv: Any, **kwargs: Any) -> None:
    srgb_profile_url = NSURL.fileURLWithPath_(
        "/System/Library/ColorSync/Profiles/sRGB Profile.icc"
    )
    set_all_displays(srgb_profile_url)
