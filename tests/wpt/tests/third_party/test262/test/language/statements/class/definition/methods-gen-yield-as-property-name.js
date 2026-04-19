// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` may be used as a literal property name in an object literal
    within generator function bodies.
features: [generators]
es6id: 12.1.1
---*/

var result;
class A {
  *g() {
    ({  yield: 1 });
  }
}

result = A.prototype.g().next();
assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
