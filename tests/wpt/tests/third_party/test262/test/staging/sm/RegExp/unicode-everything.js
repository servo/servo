// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- everything Atom.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== standalone ====

assert.compareArray(/./u.exec("ABC"),
              ["A"]);
assert.compareArray(/./u.exec("\u{1F438}BC"),
              ["\u{1F438}"]);

assert.compareArray(/./u.exec("\uD83D\uDBFF"),
              ["\uD83D"]);
assert.compareArray(/./u.exec("\uD83D\uDC00"),
              ["\uD83D\uDC00"]);
assert.compareArray(/./u.exec("\uD83D\uDFFF"),
              ["\uD83D\uDFFF"]);
assert.compareArray(/./u.exec("\uD83D\uE000"),
              ["\uD83D"]);
assert.compareArray(/./u.exec("\uD83D"),
              ["\uD83D"]);
assert.compareArray(/./u.exec("\uD83DA"),
              ["\uD83D"]);

assert.compareArray(/./u.exec("\uD7FF\uDC38"),
              ["\uD7FF"]);
assert.compareArray(/./u.exec("\uD800\uDC38"),
              ["\uD800\uDC38"]);
assert.compareArray(/./u.exec("\uDBFF\uDC38"),
              ["\uDBFF\uDC38"]);
assert.compareArray(/./u.exec("\uDC00\uDC38"),
              ["\uDC00"]);
assert.compareArray(/./u.exec("\uDC38"),
              ["\uDC38"]);
assert.compareArray(/./u.exec("A\uDC38"),
              ["A"]);

assert.compareArray(/.A/u.exec("\uD7FF\uDC38A"),
              ["\uDC38A"]);
assert.compareArray(/.A/u.exec("\uD800\uDC38A"),
              ["\uD800\uDC38A"]);
assert.compareArray(/.A/u.exec("\uDBFF\uDC38A"),
              ["\uDBFF\uDC38A"]);
assert.compareArray(/.A/u.exec("\uDC00\uDC38A"),
              ["\uDC38A"]);

// ==== leading multiple ====

assert.compareArray(/.*A/u.exec("\u{1F438}\u{1F438}\u{1F438}A"),
              ["\u{1F438}\u{1F438}\u{1F438}A"]);

// ==== trailing multiple ====

assert.compareArray(/A.*/u.exec("A\u{1F438}\u{1F438}\u{1F438}"),
              ["A\u{1F438}\u{1F438}\u{1F438}"]);
