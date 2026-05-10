// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.map
description: >
  Iterator.prototype.map returns an empty iterator when the iterator has already been exhausted
info: |
  %Iterator.prototype%.map ( mapper )

features: [iterator-helpers]
flags: []
---*/
let iterator = (function* () {})();

let { value, done } = iterator.next();
assert.sameValue(value, undefined);
assert.sameValue(done, true);

iterator = iterator.map(() => 0);
({ value, done } = iterator.next());
assert.sameValue(value, undefined);
assert.sameValue(done, true);
