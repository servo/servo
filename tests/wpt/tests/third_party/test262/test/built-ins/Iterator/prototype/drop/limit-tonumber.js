// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Converts the limit argument to a Number using ToNumber and valueOf/toString.
info: |
  %Iterator.prototype%.drop ( limit )

  2. Let numLimit be ? ToNumber(limit).

features: [iterator-helpers]
---*/
function* g() {
  yield 1;
  yield 2;
}

{
  let iterator = g();
  let { value, done } = iterator
    .drop({
      valueOf: function () {
        return 1;
      },
    })
    .next();
  assert.sameValue(value, 2);
  assert.sameValue(done, false);
}

{
  let iterator = g();
  let { value, done } = iterator.drop([]).drop([1]).next();
  assert.sameValue(value, 2);
  assert.sameValue(done, false);
}
