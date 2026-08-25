// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Null in character class in RegExp with unicode flag.
info: bugzilla.mozilla.org/show_bug.cgi?id=1279467
esid: pending
---*/

var m = /([\0]+)/u.exec("\u0000");
assert.sameValue(m.length, 2);
assert.sameValue(m[0], '\u0000');
assert.sameValue(m[1], '\u0000');

var m = /([\0]+)/u.exec("0");
assert.sameValue(m, null);
