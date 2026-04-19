// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: >
  Error when derived constructor returns a non-undefined value (honoring
  the Realm of the current execution context)
info: |
  [...]
  13. If result.[[Type]] is return, then
      a. If Type(result.[[Value]]) is Object, return
         NormalCompletion(result.[[Value]]).
      b. If kind is "base", return NormalCompletion(thisArgument).
      c. If result.[[Value]] is not undefined, throw a TypeError exception.
  [...]
features: [cross-realm, class]
---*/

var C = $262.createRealm().global.eval(
  '0, class extends Object {' +
  '  constructor() {' +
  '    return null;' +
  '  }' +
  '}'
);

assert.throws(TypeError, function() {
  new C();
});
