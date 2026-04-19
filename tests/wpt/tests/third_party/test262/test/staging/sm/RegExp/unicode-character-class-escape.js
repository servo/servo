// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- CharacterClassEscape.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// BMP

assert.compareArray(/\d+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["0123456789"]);
assert.compareArray(/\D+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ"]);

assert.compareArray(/\s+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\t\r\n\v\x0c\xa0\uFEFF"]);
assert.compareArray(/\S+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ0123456789_"]);

assert.compareArray(/\w+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ0123456789_"]);
assert.compareArray(/\W+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\t\r\n\v\x0c\xa0\uFEFF*"]);

assert.compareArray(/\n+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\n"]);

assert.compareArray(/[\d]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["0123456789"]);
assert.compareArray(/[\D]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ"]);

assert.compareArray(/[\s]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\t\r\n\v\x0c\xa0\uFEFF"]);
assert.compareArray(/[\S]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ0123456789_"]);

assert.compareArray(/[\w]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["abcxyzABCXYZ0123456789_"]);
assert.compareArray(/[\W]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\t\r\n\v\x0c\xa0\uFEFF*"]);

assert.compareArray(/[\n]+/u.exec("abcxyzABCXYZ0123456789_\t\r\n\v\x0c\xa0\uFEFF*"),
              ["\n"]);

// non-BMP

function testNonBMP(re) {
  assert.compareArray(re.exec("\uD83D\uDBFF"),
                ["\uD83D"]);
  assert.compareArray(re.exec("\uD83D\uDC00"),
                ["\uD83D\uDC00"]);
  assert.compareArray(re.exec("\uD83D\uDFFF"),
                ["\uD83D\uDFFF"]);
  assert.compareArray(re.exec("\uD83D\uE000"),
                ["\uD83D"]);

  assert.compareArray(re.exec("\uD7FF\uDC38"),
                ["\uD7FF"]);
  assert.compareArray(re.exec("\uD800\uDC38"),
                ["\uD800\uDC38"]);
  assert.compareArray(re.exec("\uDBFF\uDC38"),
                ["\uDBFF\uDC38"]);
  assert.compareArray(re.exec("\uDC00\uDC38"),
                ["\uDC00"]);
}

testNonBMP(/\D/u);
testNonBMP(/\S/u);
testNonBMP(/\W/u);

testNonBMP(/[\D]/u);
testNonBMP(/[\S]/u);
testNonBMP(/[\W]/u);
