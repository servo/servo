// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every returns true when the predicate returns truthy for all iterated values
info: |
  %Iterator.prototype%.every ( predicate )

  4.b. If next is false, return true.

features: [iterator-helpers]
flags: []
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
  yield 4;
}

let result = g().every(() => true);
assert.sameValue(result, true);
