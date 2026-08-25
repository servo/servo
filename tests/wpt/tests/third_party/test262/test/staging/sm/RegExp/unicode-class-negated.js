// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- negated CharacterClass.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// ==== BMP ====

assert.compareArray(/[^A]/u.exec("ABC"),
              ["B"]);
assert.compareArray(/[^A]/u.exec("A\u{1F438}C"),
              ["\u{1F438}"]);
assert.compareArray(/[^A]/u.exec("A\uD83DC"),
              ["\uD83D"]);
assert.compareArray(/[^A]/u.exec("A\uDC38C"),
              ["\uDC38"]);

assert.compareArray(/[^\uE000]/u.exec("\uE000\uE001"),
              ["\uE001"]);
assert.compareArray(/[^\uE000]/u.exec("\uE000\u{1F438}"),
              ["\u{1F438}"]);
assert.compareArray(/[^\uE000]/u.exec("\uE000\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[^\uE000]/u.exec("\uE000\uDC38"),
              ["\uDC38"]);

// ==== non-BMP ====

assert.compareArray(/[^\u{1F438}]/u.exec("\u{1F438}A"),
              ["A"]);
assert.compareArray(/[^\u{1F438}]/u.exec("\u{1F438}\u{1F439}"),
              ["\u{1F439}"]);
assert.compareArray(/[^\u{1F438}]/u.exec("\u{1F438}\uD83D"),
              ["\uD83D"]);
assert.compareArray(/[^\u{1F438}]/u.exec("\u{1F438}\uDC38"),
              ["\uDC38"]);

// ==== lead-only ====

assert.compareArray(/[^\uD83D]/u.exec("\u{1F438}A"),
              ["\u{1F438}"]);
assert.compareArray(/[^\uD83D]/u.exec("\uD83D\uDBFF"),
              ["\uDBFF"]);
assert.compareArray(/[^\uD83D]/u.exec("\uD83D\uDC00"),
              ["\uD83D\uDC00"]);
assert.compareArray(/[^\uD83D]/u.exec("\uD83D\uDFFF"),
              ["\uD83D\uDFFF"]);
assert.compareArray(/[^\uD83D]/u.exec("\uD83D\uE000"),
              ["\uE000"]);

// ==== trail-only ====

assert.compareArray(/[^\uDC38]/u.exec("\u{1F438}A"),
              ["\u{1F438}"]);
assert.compareArray(/[^\uDC38]/u.exec("\uD7FF\uDC38"),
              ["\uD7FF"]);
assert.compareArray(/[^\uDC38]/u.exec("\uD800\uDC38"),
              ["\uD800\uDC38"]);
assert.compareArray(/[^\uDC38]/u.exec("\uDBFF\uDC38"),
              ["\uDBFF\uDC38"]);
assert.compareArray(/[^\uDC38]/u.exec("\uDC00\uDC38"),
              ["\uDC00"]);
