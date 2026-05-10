// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  Removes entries from this iterator, specified by limit argument.
info: |
  %Iterator.prototype%.drop ( limit )

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 1;
  yield 2;
}

let iterator = g().drop(2);
let { value, done } = iterator.next();

assert.sameValue(value, undefined);
assert.sameValue(done, true);
