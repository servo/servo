/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx::text::util::is_cjk;

#[test]
fn test_is_cjk() {
    // Test characters from different CJK blocks
    assert_eq!(is_cjk('〇'), true);
    assert_eq!(is_cjk('㐀'), true);
    assert_eq!(is_cjk('あ'), true);
    assert_eq!(is_cjk('ア'), true);
    assert_eq!(is_cjk('㆒'), true);
    assert_eq!(is_cjk('ㆣ'), true);
    assert_eq!(is_cjk('龥'), true);
    assert_eq!(is_cjk('𰾑'), true);
    assert_eq!(is_cjk('𰻝'), true);

    // Test characters from outside CJK blocks
    assert_eq!(is_cjk('a'), false);
    assert_eq!(is_cjk('🙂'), false);
    assert_eq!(is_cjk('©'), false);
}
