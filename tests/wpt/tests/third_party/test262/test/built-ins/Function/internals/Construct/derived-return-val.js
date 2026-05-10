// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-construct-argumentslist-newtarget
description: Error when derived constructor returns a non-undefined value
info: |
  [...]
  13. If result.[[Type]] is return, then
      a. If Type(result.[[Value]]) is Object, return
         NormalCompletion(result.[[Value]]).
      b. If kind is "base", return NormalCompletion(thisArgument).
      c. If result.[[Value]] is not undefined, throw a TypeError exception.
  [...]
features: [class]
---*/

class C extends Object {
  constructor() {
    return null;
  }
}

assert.throws(TypeError, function() {
  new C();
});
