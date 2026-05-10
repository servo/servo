// Copyright (c) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-getidentifierreference
es6id: 8.1.2.1
description: >
  Behavior when binding query produces an abrupt completion
info: |
  [...]
  2. Let envRec be lex's EnvironmentRecord.
  3. Let exists be ? envRec.HasBinding(name).

  8.1.1.2.1 HasBinding

  1. Let envRec be the object Environment Record for which the method was
     invoked.
  2. Let bindings be the binding object for envRec.
  3. Let foundBinding be ? HasProperty(bindings, N).
flags: [noStrict]
features: [Proxy]
---*/

var thrower = new Proxy({}, {
  has: function(_, name) {
    if (name === 'test262') {
      throw new Test262Error();
    }
  }
});

with (thrower) {
  assert.throws(Test262Error, function() {
    test262;
  });
}
