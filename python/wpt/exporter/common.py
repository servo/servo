# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# pylint: disable=missing-docstring

UPSTREAMABLE_PATH = "tests/wpt/tests/"
NO_SYNC_SIGNAL = "[no-wpt-sync]"

OPENED_NEW_UPSTREAM_PR = (
    "🤖 Opened new upstream WPT pull request ({upstream_pr}) "
    "with upstreamable changes."
)
UPDATED_EXISTING_UPSTREAM_PR = (
    "📝 Transplanted new upstreamable changes to existing "
    "upstream WPT pull request ({upstream_pr})."
)
UPDATED_TITLE_IN_EXISTING_UPSTREAM_PR = (
    "✍ Updated existing upstream WPT pull request ({upstream_pr}) title and body."
)
CLOSING_EXISTING_UPSTREAM_PR = (
    "🤖 This change no longer contains upstreamable changes to WPT; closed existing "
    "upstream pull request ({upstream_pr})."
)
NO_UPSTREAMBLE_CHANGES_COMMENT = (
    "👋 Downstream pull request ({servo_pr}) no longer contains any upstreamable "
    "changes. Closing pull request without merging."
)
COULD_NOT_APPLY_CHANGES_DOWNSTREAM_COMMENT = (
    "🛠 These changes could not be applied onto the latest upstream WPT. "
    "Servo's copy of the Web Platform Tests may be out of sync."
)
COULD_NOT_APPLY_CHANGES_UPSTREAM_COMMENT = (
    "🛠 Changes from the source pull request ({servo_pr}) can no longer be "
    "cleanly applied. Waiting for a new version of these changes downstream."
)
COULD_NOT_MERGE_CHANGES_DOWNSTREAM_COMMENT = (
    "⛔ Failed to properly merge the upstream pull request ({upstream_pr}). "
    "Please address any CI issues and try to merge manually."
)
COULD_NOT_MERGE_CHANGES_UPSTREAM_COMMENT = (
    "⛔ The downstream PR has merged ({servo_pr}), but these changes could not "
    "be merged properly. Please address any CI issues and try to merge manually."
)


def wpt_branch_name_from_servo_pr_number(servo_pr_number):
    return f"servo_export_{servo_pr_number}"
