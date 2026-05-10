// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-ownpropertykeys
description: >
    Trap returns abrupt.
info: |
    [[OwnPropertyKeys]] ( )

    ...
    7. Let trapResultArray be ? Call(trap, handler, « target »).
    8. Let trapResult be ? CreateListFromArrayLike(trapResultArray, « String, Symbol »).

features: [Proxy]
---*/

var p = new Proxy({}, {
  ownKeys: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.keys(p);
});
