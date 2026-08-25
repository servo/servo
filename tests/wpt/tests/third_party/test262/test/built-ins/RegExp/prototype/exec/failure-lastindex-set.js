// Copyright (C) 2021 Richard Gibson. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    lastIndex is set to 0 after exhausting the string when global and/or sticky are set.
esid: sec-regexpbuiltinexec
info: |
    RegExpBuiltinExec (
      _R_: an initialized RegExp instance,
      _S_: a String,
    )
    ...
    1. Let _length_ be the number of code units in _S_.
    2. Let _lastIndex_ be ℝ(? ToLength(? Get(_R_, *"lastIndex"*))).
    3. Let _flags_ be _R_.[[OriginalFlags]].
    4. If _flags_ contains *"g"*, let _global_ be *true*; else let _global_ be *false*.
    5. If _flags_ contains *"y"*, let _sticky_ be *true*; else let _sticky_ be *false*.
    ...
    9. Let _matchSucceeded_ be *false*.
    10. Repeat, while _matchSucceeded_ is *false*,
      a. If _lastIndex_ &gt; _length_, then
        i. If _global_ is *true* or _sticky_ is *true*, then
          1. Perform ? Set(_R_, *"lastIndex"*, *+0*<sub>𝔽</sub>, *true*).
        ii. Return *null*.
features: [exponentiation]
---*/

var R_g = /./g, R_y = /./y, R_gy = /./gy;

var S = "test";

var lastIndex;
var bigLastIndexes = [
  Infinity,
  Number.MAX_VALUE,
  Number.MAX_SAFE_INTEGER,
  Number.MAX_SAFE_INTEGER - 1,
  2**32 + 4,
  2**32 + 3,
  2**32 + 2,
  2**32 + 1,
  2**32,
  2**32 - 1,
  5
];
for ( var i = 0; i < bigLastIndexes.length; i++ ) {
  lastIndex = bigLastIndexes[i];
  R_g.lastIndex = lastIndex;
  R_y.lastIndex = lastIndex;
  R_gy.lastIndex = lastIndex;

  assert.sameValue(R_g.exec(S), null,
      "global RegExp instance must fail to match against '" + S +
      "' at lastIndex " + lastIndex);
  assert.sameValue(R_y.exec(S), null,
      "sticky RegExp instance must fail to match against '" + S +
      "' at lastIndex " + lastIndex);
  assert.sameValue(R_gy.exec(S), null,
      "global sticky RegExp instance must fail to match against '" + S +
      "' at lastIndex " + lastIndex);

  assert.sameValue(R_g.lastIndex, 0,
      "global RegExp instance lastIndex must be reset after " + lastIndex +
      " exceeds string length");
  assert.sameValue(R_y.lastIndex, 0,
      "sticky RegExp instance lastIndex must be reset after " + lastIndex +
      " exceeds string length");
  assert.sameValue(R_gy.lastIndex, 0,
      "global sticky RegExp instance lastIndex must be reset after " + lastIndex +
      " exceeds string length");
}
