import argparse
import sys
from typing import Any, NewType, Optional, Tuple

from Cocoa import NSURL
from ColorSync import (
    CGDisplayCreateUUIDFromDisplayID,
    ColorSyncDeviceSetCustomProfiles,
    kColorSyncDeviceDefaultProfileID,
    kColorSyncDisplayDeviceClass,
)
from Quartz import (
    CGBeginDisplayConfiguration,
    CGCancelDisplayConfiguration,
    CGCompleteDisplayConfiguration,
    CGConfigureDisplayWithDisplayMode,
    CGDisplayCopyAllDisplayModes,
    CGDisplayCopyDisplayMode,
    CGDisplayModeGetHeight,
    CGDisplayModeGetIOFlags,
    CGDisplayModeGetPixelHeight,
    CGDisplayModeGetPixelWidth,
    CGDisplayModeGetRefreshRate,
    CGDisplayModeGetWidth,
    CGDisplayModeIsUsableForDesktopGUI,
    CGDisplayModeRef,
    CGGetOnlineDisplayList,
    kCGConfigurePermanently,
    kCGErrorSuccess,
)

# Display mode flags
kDisplayModeDefaultFlag = 0x00000004  # noqa: N816

# Create a new type for display IDs
CGDirectDisplayID = NewType("CGDirectDisplayID", int)


def get_pixel_size(mode: CGDisplayModeRef) -> Tuple[int, int]:
    return (CGDisplayModeGetPixelWidth(mode), CGDisplayModeGetPixelHeight(mode))


def get_size(mode: CGDisplayModeRef) -> Tuple[int, int]:
    return (CGDisplayModeGetWidth(mode), CGDisplayModeGetHeight(mode))


def calculate_mode_similarity_score(
    mode: CGDisplayModeRef, current_mode: CGDisplayModeRef
) -> int:
    current_size = get_size(current_mode)
    current_pixel_size = get_pixel_size(current_mode)
    current_refresh_rate = CGDisplayModeGetRefreshRate(current_mode)
    current_flags = CGDisplayModeGetIOFlags(current_mode)

    size = get_size(mode)
    pixel_size = get_pixel_size(mode)
    refresh_rate = CGDisplayModeGetRefreshRate(mode)
    flags = CGDisplayModeGetIOFlags(mode)

    differences = 0

    if size != current_size:
        differences += 1
    if pixel_size != current_pixel_size:
        differences += 1
    if refresh_rate != current_refresh_rate:
        differences += 1

    # Count how many individual flags are changing (XOR then count bits)
    changed_flags = flags ^ current_flags
    if sys.version_info >= (3, 10):
        differences += changed_flags.bit_count()
    else:
        differences += bin(changed_flags).count("1")

    return differences


def find_best_unscaled_mode(display_id: CGDirectDisplayID) -> CGDisplayModeRef:
    current_mode: Optional[CGDisplayModeRef] = CGDisplayCopyDisplayMode(display_id)

    # If we already have an unscaled mode, we're done.
    if current_mode and (
        get_size(current_mode) == get_pixel_size(current_mode)
    ):
        return current_mode

    all_modes = CGDisplayCopyAllDisplayModes(display_id, None)
    if not all_modes:
        raise Exception("No display modes")

    # If we don't have a current mode, use the default mode instead.
    if not current_mode:
        default_modes = [
            m for m in all_modes if CGDisplayModeGetIOFlags(m) & kDisplayModeDefaultFlag
        ]
        if not default_modes:
            raise Exception("No default display mode found")
        current_mode = default_modes[0]
        assert current_mode is not None

        if get_size(current_mode) == get_pixel_size(current_mode):
            return current_mode

    candidates = [
        m
        for m in all_modes
        if CGDisplayModeIsUsableForDesktopGUI(m) and get_size(m) == get_pixel_size(m)
    ]
    if not candidates:
        raise Exception("No suitable display modes")

    same_size_candidates = [
        m for m in candidates if get_size(m) == get_size(current_mode)
    ]
    same_pixel_size_candidates = [
        m for m in candidates if get_pixel_size(m) == get_pixel_size(current_mode)
    ]

    if same_size_candidates:
        candidates = same_size_candidates
    elif same_pixel_size_candidates:
        candidates = same_pixel_size_candidates

    return min(
        candidates,
        key=lambda m: calculate_mode_similarity_score(m, current_mode),
    )


def set_color_profiles(profile_url: NSURL, *, dry_run: bool = False) -> bool:
    max_displays = 10

    (err, display_ids, display_count) = CGGetOnlineDisplayList(max_displays, None, None)
    if err != kCGErrorSuccess:
        raise ValueError(err)

    display_uuids = [CGDisplayCreateUUIDFromDisplayID(d) for d in display_ids]

    for display_id, display_uuid in zip(display_ids, display_uuids):
        if dry_run:
            print(
                f"Would set color profile for display {display_id} to {profile_url.path()}"
            )
        else:
            profile_info = {kColorSyncDeviceDefaultProfileID: profile_url}
            success = ColorSyncDeviceSetCustomProfiles(
                kColorSyncDisplayDeviceClass,
                display_uuid,
                profile_info,
            )
            if not success:
                raise Exception(f"failed to set profile on {display_uuid}")
            print(f"Set color profile for display {display_id}")

    return True


def set_display_modes(*, dry_run: bool = False) -> bool:
    max_displays = 10

    err, display_ids, display_count = CGGetOnlineDisplayList(max_displays, None, None)
    if err != kCGErrorSuccess:
        raise ValueError(err)

    if dry_run:
        for display_id in display_ids:
            best_mode = find_best_unscaled_mode(display_id)
            best_size = get_size(best_mode)
            print(f"Would change display {display_id} to {best_size}")
        return True

    err, config_ref = CGBeginDisplayConfiguration(None)
    if err != kCGErrorSuccess:
        raise Exception("Failed to begin display configuration")

    try:
        for display_id in display_ids:
            best_mode = find_best_unscaled_mode(display_id)
            best_size = get_size(best_mode)

            err = CGConfigureDisplayWithDisplayMode(
                config_ref, display_id, best_mode, None
            )
            if err != kCGErrorSuccess:
                raise Exception(
                    f"Failed to configure mode for display {display_id}: {err}"
                )

            print(f"Configured display {display_id} mode to {best_size}")

    except Exception:
        CGCancelDisplayConfiguration(config_ref)
        raise

    else:
        err = CGCompleteDisplayConfiguration(config_ref, kCGConfigurePermanently)
        if err != kCGErrorSuccess:
            raise Exception(f"Failed to complete display configuration: {err}")

        print("Display configuration applied permanently")

    return True


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without making changes",
    )
    parser.add_argument(
        "--no-color-profile",
        action="store_false",
        dest="color_profile",
        help="Don't set color profiles",
    )
    parser.add_argument(
        "--no-display-mode",
        action="store_false",
        dest="display_mode",
        help="Don't set display mode",
    )
    parser.add_argument(
        "--profile-path",
        default="/System/Library/ColorSync/Profiles/sRGB Profile.icc",
        help="Path to color profile to use (default: sRGB)",
    )
    return parser


def run(venv: Any, **kwargs: Any) -> None:
    profile_url = NSURL.fileURLWithPath_(kwargs["profile_path"])
    dry_run = kwargs["dry_run"]

    if kwargs["color_profile"]:
        set_color_profiles(profile_url, dry_run=dry_run)

    if kwargs["display_mode"]:
        set_display_modes(dry_run=dry_run)
