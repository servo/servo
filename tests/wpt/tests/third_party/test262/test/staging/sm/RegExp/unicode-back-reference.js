// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- back reference should not match lead surrogate that has corresponding trail surrogate.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// The last character of back reference is not a surrogate.
assert.compareArray(/foo(.+)bar\1/u.exec("fooAbarA\uDC00"),
              ["fooAbarA", "A"]);
assert.compareArray(/foo(.+)bar\1/u.exec("fooAbarA\uD834"),
              ["fooAbarA", "A"]);
assert.compareArray(/foo(.+)bar\1/u.exec("fooAbarAA"),
              ["fooAbarA", "A"]);
assert.compareArray(/foo(.+)bar\1/u.exec("fooAbarA"),
              ["fooAbarA", "A"]);

// The last character of back reference is a lead surrogate.
assert.sameValue(/foo(.+)bar\1/u.exec("foo\uD834bar\uD834\uDC00"), null);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uD834bar\uD834\uD834"),
              ["foo\uD834bar\uD834", "\uD834"]);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uD834bar\uD834A"),
              ["foo\uD834bar\uD834", "\uD834"]);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uD834bar\uD834"),
              ["foo\uD834bar\uD834", "\uD834"]);

// The last character of back reference is a trail surrogate.
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uDC00bar\uDC00\uDC00"),
              ["foo\uDC00bar\uDC00", "\uDC00"]);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uDC00bar\uDC00\uD834"),
              ["foo\uDC00bar\uDC00", "\uDC00"]);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uDC00bar\uDC00A"),
              ["foo\uDC00bar\uDC00", "\uDC00"]);
assert.compareArray(/foo(.+)bar\1/u.exec("foo\uDC00bar\uDC00"),
              ["foo\uDC00bar\uDC00", "\uDC00"]);

// Pattern should not match to surrogate pair partially.
assert.sameValue(/^(.+)\1$/u.exec("\uDC00foobar\uD834\uDC00foobar\uD834"), null);
