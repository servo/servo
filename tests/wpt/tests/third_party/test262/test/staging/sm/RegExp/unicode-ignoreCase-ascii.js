// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  Implement RegExp unicode flag -- ignoreCase flag with non-ascii to ascii map.
info: bugzilla.mozilla.org/show_bug.cgi?id=1135377
esid: pending
---*/

// LATIN CAPITAL LETTER Y WITH DIAERESIS
assert.compareArray(/\u0178/iu.exec("\u00FF"),
              ["\u00FF"]);
assert.compareArray(/\u00FF/iu.exec("\u0178"),
              ["\u0178"]);

// LATIN SMALL LETTER LONG S
assert.compareArray(/\u017F/iu.exec("S"),
              ["S"]);
assert.compareArray(/\u017F/iu.exec("s"),
              ["s"]);
assert.compareArray(/S/iu.exec("\u017F"),
              ["\u017F"]);
assert.compareArray(/s/iu.exec("\u017F"),
              ["\u017F"]);

// LATIN CAPITAL LETTER SHARP S
assert.compareArray(/\u1E9E/iu.exec("\u00DF"),
              ["\u00DF"]);
assert.compareArray(/\u00DF/iu.exec("\u1E9E"),
              ["\u1E9E"]);

// KELVIN SIGN
assert.compareArray(/\u212A/iu.exec("K"),
              ["K"]);
assert.compareArray(/\u212A/iu.exec("k"),
              ["k"]);
assert.compareArray(/K/iu.exec("\u212A"),
              ["\u212A"]);
assert.compareArray(/k/iu.exec("\u212A"),
              ["\u212A"]);

// ANGSTROM SIGN
assert.compareArray(/\u212B/iu.exec("\u00E5"),
              ["\u00E5"]);
assert.compareArray(/\u00E5/iu.exec("\u212B"),
              ["\u212B"]);
