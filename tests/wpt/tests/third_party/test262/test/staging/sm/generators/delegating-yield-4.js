// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-generators-shell.js]
description: |
  pending
esid: pending
---*/
// With yield*, inner and outer iterators can be invoked separately.

function* g(n) { for (var i=0; i<n; i++) yield i; }
function* delegate(iter) { return yield* iter; }

var inner = g(20);
var outer1 = delegate(inner);
var outer2 = delegate(inner);

assertIteratorNext(outer1, 0);
assertIteratorNext(outer2, 1);
assertIteratorNext(inner, 2);
assertIteratorNext(outer1, 3);
assertIteratorNext(outer2, 4);
assertIteratorNext(inner, 5);

