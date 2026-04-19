// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.every
description: >
  Iterator.prototype.every returns true when the iterator has already been exhausted
info: |
  %Iterator.prototype%.every ( predicate )

  4.a. Let next be ? IteratorStep(iterated).
  4.b. If next is false, return true.

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {})();

let { value, done } = iterator.next();
assert.sameValue(value, undefined);
assert.sameValue(done, true);

let result = iterator.every(() => true);
assert.sameValue(result, true);

result = iterator.every(() => false);
assert.sameValue(result, true);
