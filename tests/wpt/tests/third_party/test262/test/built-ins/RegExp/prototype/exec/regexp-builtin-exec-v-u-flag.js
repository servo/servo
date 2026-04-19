// Copyright (C) 2024 Tan Meng. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexpbuiltinexec
description: RegExpBuiltinExec behavior with 'u' and 'v'flags
features: [regexp-v-flag, regexp-unicode-property-escapes]
includes: [compareArray.js]
---*/

const text = '𠮷a𠮷b𠮷';

function doExec(regex) {
  const result = regex.exec(text);
  return result ? [result[0], result.index] : null;
}

assert.compareArray(doExec(/𠮷/), ["𠮷", 0], "Basic exec without v flag");

assert.compareArray(doExec(/𠮷/u), ["𠮷", 0], "Exec with u flag");
assert.compareArray(doExec(/\p{Script=Han}/u), ["𠮷", 0], "Unicode property escapes with u flag");
assert.compareArray(doExec(/./u), ["𠮷", 0], "Dot with u flag");

assert.compareArray(doExec(/𠮷/v), ["𠮷", 0], "Exec with v flag");
assert.compareArray(doExec(/\p{Script=Han}/v), ["𠮷", 0], "Unicode property escapes with v flag");
assert.compareArray(doExec(/./v), ["𠮷", 0], "Dot with v flag");

assert.compareArray(doExec(/\p{ASCII}/u), ["a", 2], "ASCII with u flag");
assert.compareArray(doExec(/\p{ASCII}/v), ["a", 2], "ASCII with v flag");

assert.sameValue(doExec(/x/u), null, "Non-matching regex with u flag");
assert.sameValue(doExec(/x/v), null, "Non-matching regex with v flag");

const regexWithGroupsU = /(\p{Script=Han})(.)/u;
const resultWithGroupsU = regexWithGroupsU.exec(text);
assert.sameValue(resultWithGroupsU[1], "𠮷", "Capture group 1 with u flag");
assert.sameValue(resultWithGroupsU[2], "a", "Capture group 2 with u flag");
assert.sameValue(resultWithGroupsU.index, 0, "Match index for groups with u flag");

const regexWithGroupsV = /(\p{Script=Han})(.)/v;
const resultWithGroupsV = regexWithGroupsV.exec(text);
assert.sameValue(resultWithGroupsV[1], "𠮷", "Capture group 1 with v flag");
assert.sameValue(resultWithGroupsV[2], "a", "Capture group 2 with v flag");
assert.sameValue(resultWithGroupsV.index, 0, "Match index for groups with v flag");

const complexText = 'a\u{20BB7}b\u{10FFFF}c';
assert.compareArray(/\P{ASCII}/u.exec(complexText), ["\u{20BB7}"], "Non-ASCII with u flag");
assert.compareArray(/\P{ASCII}/v.exec(complexText), ["\u{20BB7}"], "Non-ASCII with v flag");
