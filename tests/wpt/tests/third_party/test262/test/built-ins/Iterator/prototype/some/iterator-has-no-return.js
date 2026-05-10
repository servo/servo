// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  The underlying iterator is sometimes unable to be closed (has no return method)
info: |
  %Iterator.prototype%.some ( predicate )

features: [iterator-helpers]
flags: []
---*/
let iterator = [1, 2, 3, 4, 5][Symbol.iterator]();

assert.sameValue(iterator.return, undefined);

let ret = iterator.some(v => v > 3);

assert.sameValue(ret, true);

let { done, value } = iterator.next();
assert.sameValue(done, false);
assert.sameValue(value, 5);

({ done, value } = iterator.next());
assert.sameValue(done, true);
assert.sameValue(value, undefined);
