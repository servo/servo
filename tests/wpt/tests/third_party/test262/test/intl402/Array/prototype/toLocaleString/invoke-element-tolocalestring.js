// Copyright (C) 2022 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sup-array.prototype.tolocalestring
description: >
    The toLocaleString method of each non-undefined non-null element
    must be called with two arguments.
info: |
    Array.prototype.toLocaleString ( [ _locales_ [ , _options_ ] ] )

    4. Let _R_ be the empty String.
    5. Let _k_ be 0.
    6. Repeat, while _k_ &lt; _len_,
      a. If _k_ &gt; 0, then
        i. Set _R_ to the string-concatenation of _R_ and _separator_.
      b. Let _nextElement_ be ? Get(_array_, ! ToString(_k_)).
      c. If _nextElement_ is not *undefined* or *null*, then
        i. Let _S_ be ? ToString(? Invoke(_nextElement_, *"toLocaleString"*, &laquo; _locales_, _options_ &raquo;)).
        ii. Set _R_ to the string-concatenation of _R_ and _S_.
      d. Increase _k_ by 1.
    7. Return _R_.
includes: [compareArray.js]
---*/

const unique = {
  toString() {
    return "<sentinel object>";
  }
};

const testCases = [
  { label: "no arguments", args: [], expectedArgs: [undefined, undefined] },
  { label: "undefined locale", args: [undefined], expectedArgs: [undefined, undefined] },
  { label: "string locale", args: ["ar"], expectedArgs: ["ar", undefined] },
  { label: "object locale", args: [unique], expectedArgs: [unique, undefined] },
  { label: "undefined locale and options", args: [undefined, unique], expectedArgs: [undefined, unique] },
  { label: "string locale and options", args: ["zh", unique], expectedArgs: ["zh", unique] },
  { label: "object locale and options", args: [unique, unique], expectedArgs: [unique, unique] },
  { label: "extra arguments", args: [unique, unique, unique], expectedArgs: [unique, unique] },
];

for (const { label, args, expectedArgs } of testCases) {
  assert.sameValue([undefined].toLocaleString(...args), "",
    `must skip undefined elements when provided ${label}`);
}
for (const { label, args, expectedArgs } of testCases) {
  assert.sameValue([null].toLocaleString(...args), "",
    `must skip null elements when provided ${label}`);
}

for (const { label, args, expectedArgs } of testCases) {
  const spy = {
    toLocaleString(...receivedArgs) {
      assert.compareArray(receivedArgs, expectedArgs,
        `must invoke element toLocaleString with expected arguments when provided ${label}`);
      return "ok";
    }
  };
  assert.sameValue([spy].toLocaleString(...args), "ok");
}
