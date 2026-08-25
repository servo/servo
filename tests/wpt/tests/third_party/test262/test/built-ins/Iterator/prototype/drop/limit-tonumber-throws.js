// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Throws a RangeError exception when limit argument valueOf throws.
info: |
  %Iterator.prototype%.drop ( limit )

  2. Let numLimit be ? ToNumber(limit).

features: [iterator-helpers]
---*/
let iterator = (function* () {})();

assert.throws(Test262Error, () => {
  iterator.drop({
    valueOf: function () {
      throw new Test262Error();
    },
  });
});
