// Copyright (C) 2018 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-literals-string-literals
description: >
  U+2028 LINE SEPARATOR can appear in string literals.
info: |
  11.8.4 String Literals

  All code points may appear literally in a string literal except for the
  closing quote code points, U+005C (REVERSE SOLIDUS), U+000D (CARRIAGE RETURN),
  and U+000A (LINE FEED).
features: [json-superset]
---*/

// U+2028 in strings; UTF8(0x2028) = 0xE2 0x80 0xA8
assert.sameValue(" ", "\u2028");
