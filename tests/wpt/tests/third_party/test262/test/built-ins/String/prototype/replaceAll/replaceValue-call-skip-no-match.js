// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.replaceall
description: >
  replaceValue is not called if there isn't a match
info: |
  String.prototype.replaceAll ( searchValue, replaceValue )

  ...
  5. Let functionalReplace be IsCallable(replaceValue).
  ...
  14. For each position in matchPositions, do
    a. If functionalReplace is true, then
      i. Let replacement be ? ToString(? Call(replaceValue, undefined, « searchString, position, string »).
features: [String.prototype.replaceAll]
---*/

function replaceValue() {
  throw new Test262Error();
}

assert.sameValue(
  'a'.replaceAll('b', replaceValue),
  'a'
);

assert.sameValue(
  'a'.replaceAll('aa', replaceValue),
  'a'
);
