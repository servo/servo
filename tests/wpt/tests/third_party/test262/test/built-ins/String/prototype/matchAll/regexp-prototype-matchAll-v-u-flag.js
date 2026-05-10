// Copyright (C) 2024 Tan Meng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype-@@matchall
description: RegExp.prototype[@@matchAll] behavior with 'u'and 'v' flags.
features: [Symbol.matchAll, regexp-v-flag, regexp-unicode-property-escapes]
includes: [compareArray.js]
---*/

const text = '𠮷a𠮷b𠮷';

function doMatchAll(regex) {
  const result = Array.from(RegExp.prototype[Symbol.matchAll].call(regex, text));
  const matches = result.map(m => m[0]);
  const indices = result.map(m => m.index);
  return matches.concat(indices);
}

assert.sameValue(
  assert.compareArray(doMatchAll(/𠮷/g), ['𠮷', '𠮷', '𠮷', 0, 3, 6]),
  undefined,
  "Basic matchAll with g flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/𠮷/gu), ['𠮷', '𠮷', '𠮷', 0, 3, 6]),
  undefined,
  "matchAll with u flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/𠮷/gv), ['𠮷', '𠮷', '𠮷', 0, 3, 6]),
  undefined,
  "matchAll with v flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/\p{Script=Han}/gu), ['𠮷', '𠮷', '𠮷', 0, 3, 6]),
  undefined,
  "Unicode property escapes with u flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/\p{Script=Han}/gv), ['𠮷', '𠮷', '𠮷', 0, 3, 6]),
  undefined,
  "Unicode property escapes with v flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/./gu), ['𠮷', 'a', '𠮷', 'b', '𠮷', 0, 2, 3, 5, 6]),
  undefined,
  "Dot with u flag"
);

assert.sameValue(
  assert.compareArray(doMatchAll(/./gv), ['𠮷', 'a', '𠮷', 'b', '𠮷', 0, 2, 3, 5, 6]),
  undefined,
  "Dot with v flag"
);

assert.sameValue(
  doMatchAll(/(?:)/gu).length,
  12,
  "Empty matches with u flag"
);

assert.sameValue(
  doMatchAll(/(?:)/gv).length,
  12,
  "Empty matches with v flag"
);

const complexText = 'a\u{20BB7}b\u{10FFFF}c';
assert.sameValue(
  assert.compareArray(Array.from(complexText.matchAll(/\P{ASCII}/gu), m => m[0]), ['\u{20BB7}', '\u{10FFFF}']),
  undefined,
  "Non-ASCII with u flag"
);
assert.sameValue(
  assert.compareArray(Array.from(complexText.matchAll(/\P{ASCII}/gv), m => m[0]), ['\u{20BB7}', '\u{10FFFF}']),
  undefined,
  "Non-ASCII with v flag"
);
