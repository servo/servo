// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped U+002F (SOLIDUS) characters (mixed assertions)
info: |
  EncodeForRegExpEscape ( c )

  1. If c is matched by SyntaxCharacter or c is U+002F (SOLIDUS), then
    a. Return the string-concatenation of 0x005C (REVERSE SOLIDUS) and UTF16EncodeCodePoint(c).
features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('.a/b'), '\\.a\\/b', 'mixed string with solidus character is escaped correctly');
assert.sameValue(RegExp.escape('/./'), '\\/\\.\\/', 'solidus character is escaped correctly - regexp similar');
assert.sameValue(RegExp.escape('./a\\/*b+c?d^e$f|g{2}h[i]j\\k'), '\\.\\/a\\\\\\/\\*b\\+c\\?d\\^e\\$f\\|g\\{2\\}h\\[i\\]j\\\\k', 'complex string with multiple special characters is escaped correctly');
