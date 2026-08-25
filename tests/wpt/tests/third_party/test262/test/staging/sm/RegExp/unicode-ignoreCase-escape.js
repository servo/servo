// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- ignoreCase flag with character class escape.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// \W doesn't match S or K from the change in
// https://github.com/tc39/ecma262/pull/525
// (bug 1281739)

// LATIN SMALL LETTER LONG S

assert.compareArray(/\w/iu.exec("S"),
              ["S"]);
assert.compareArray(/\w/iu.exec("s"),
              ["s"]);
assert.compareArray(/\w/iu.exec("\u017F"),
              ["\u017F"]);

assert.compareArray(/[^\W]/iu.exec("S"),
              ["S"]);
assert.compareArray(/[^\W]/iu.exec("s"),
              ["s"]);
assert.compareArray(/[^\W]/iu.exec("\u017F"),
              ["\u017F"]);

assert.sameValue(/\W/iu.exec("S"),
         null);
assert.sameValue(/\W/iu.exec("s"),
         null);
assert.sameValue(/\W/iu.exec("\u017F"),
         null);

assert.sameValue(/[^\w]/iu.exec("S"),
         null);
assert.sameValue(/[^\w]/iu.exec("s"),
         null);
assert.sameValue(/[^\w]/iu.exec("\u017F"),
         null);

// KELVIN SIGN

assert.compareArray(/\w/iu.exec("k"),
              ["k"]);
assert.compareArray(/\w/iu.exec("k"),
              ["k"]);
assert.compareArray(/\w/iu.exec("\u212A"),
              ["\u212A"]);

assert.compareArray(/[^\W]/iu.exec("k"),
              ["k"]);
assert.compareArray(/[^\W]/iu.exec("k"),
              ["k"]);
assert.compareArray(/[^\W]/iu.exec("\u212A"),
              ["\u212A"]);

assert.sameValue(/\W/iu.exec("k"),
         null);
assert.sameValue(/\W/iu.exec("k"),
         null);
assert.sameValue(/\W/iu.exec("\u212A"),
         null);

assert.sameValue(/[^\w]/iu.exec("k"),
         null);
assert.sameValue(/[^\w]/iu.exec("k"),
         null);
assert.sameValue(/[^\w]/iu.exec("\u212A"),
         null);
