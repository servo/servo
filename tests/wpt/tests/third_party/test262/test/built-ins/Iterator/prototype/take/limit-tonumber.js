// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Converts the limit argument to a Number using ToNumber and valueOf/toString.
info: |
  %Iterator.prototype%.take ( limit )

  3. Let numLimit be ? ToNumber(limit).

includes: [compareArray.js]
features: [iterator-helpers]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
}

assert.compareArray(
  Array.from(
    g().take({
      valueOf: function () {
        return 0;
      },
    })
  ),
  []
);
assert.compareArray(
  Array.from(
    g().take({
      valueOf: function () {
        return 1;
      },
    })
  ),
  [0]
);
assert.compareArray(
  Array.from(
    g().take({
      valueOf: function () {
        return 2;
      },
    })
  ),
  [0, 1]
);
assert.compareArray(Array.from(g().take([1])), [0]);
assert.compareArray(
  Array.from(
    g().take({
      toString: function () {
        return '1';
      },
    })
  ),
  [0]
);
