// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5.9
description: >
    Errors thrown during IdentifierReference evaluation are forwarded to the
    runtime.
flags: [noStrict]
features: [Proxy]
---*/

var p = new Proxy({}, {
  has: function () {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  with (p) {
    ({attr});
  }
});
