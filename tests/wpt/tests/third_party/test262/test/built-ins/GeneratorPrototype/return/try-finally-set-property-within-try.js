// Copyright (C) 2022 Bo Pang. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator.prototype.return
description: >
    When a generator is paused within a `try` block of a `try..finally`
    statement, `return` should interrupt control flow as if a `return`
    statement had appeared at that location in the function body.
    The `finally` block is still evaluated, and may override the return value.
features: [generators]
---*/


var obj = { foo: 'not modified' };
function* g() {
  try {
    obj.foo = yield;
  } finally {
    return 1;
  }
}
var iter = g();
var result;

iter.next();
result = iter.return(45).value;
assert.sameValue(obj.foo, 'not modified', '`obj.foo` must not be set');
assert.sameValue(result, 1, 'finally block must supersede return value');
