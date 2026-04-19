// Copyright 2018 the V8 project authors, Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.segment
description: Verifies the string coercion in the "segment" function of the Segmenter prototype object.
info: |
    Intl.Segmenter.prototype.segment( string )

    3. Let string be ? ToString(string).
features: [Intl.Segmenter]
---*/

const tests = [
  [[], "undefined"],
  [[undefined], "undefined"],
  [[null], "null"],
  [[true], "true"],
  [[false], "false"],
  [[12], "12"],
  [[1.23], "1.23"],
  [[["a", "b"]], "a"],
  [[{}], "["], // "[object Object]"
];

const segmenter = new Intl.Segmenter("en", { "granularity": "word" });
for (const [args, expected] of tests) {
  const segments = segmenter.segment(...args);
  const actual = [...segments][0].segment;
  assert.sameValue(actual, expected, `Expected segment "${expected}", found "${actual}" for arguments ${args}`);
}

const symbol = Symbol();
assert.throws(TypeError, () => segmenter.segment(symbol));
