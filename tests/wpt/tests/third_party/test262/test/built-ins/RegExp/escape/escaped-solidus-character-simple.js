// Copyright (C) 2024 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-regexp.escape
description: Escaped U+002F (SOLIDUS) character (simple assertions)
info: |
  EncodeForRegExpEscape ( c )

  1. If c is matched by SyntaxCharacter or c is U+002F (SOLIDUS), then
    a. Return the string-concatenation of 0x005C (REVERSE SOLIDUS) and UTF16EncodeCodePoint(c).
features: [RegExp.escape]
---*/

assert.sameValue(RegExp.escape('/'), '\\/', 'solidus character is escaped correctly');
assert.sameValue(RegExp.escape('//'), '\\/\\/', 'solidus character is escaped correctly - multiple occurrences 1');
assert.sameValue(RegExp.escape('///'), '\\/\\/\\/', 'solidus character is escaped correctly - multiple occurrences 2');
assert.sameValue(RegExp.escape('////'), '\\/\\/\\/\\/', 'solidus character is escaped correctly - multiple occurrences 3');
