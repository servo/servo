// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-internalizejsonproperty
description: Abrupt completion from "holder" property access while reviving
info: |
  JSON.parse ( text [ , reviver ] )

  [...]
  7. If IsCallable(reviver) is true, then
     [...]
     e. Return ? InternalizeJSONProperty(root, rootName).

  Runtime Semantics: InternalizeJSONProperty ( holder, name)

  1. Let val be ? Get(holder, name).
---*/

var thrower = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  JSON.parse('[0,0]', function() {
    Object.defineProperty(this, '1', {
      get: thrower
    });
  });
});
