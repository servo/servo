// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
es6id: 14.4.13
description: Failure to define property for static generator method
info: |
  [...]
  10. Let desc be the PropertyDescriptor{[[Value]]: closure, [[Writable]]:
      true, [[Enumerable]]: enumerable, [[Configurable]]: true}.
  11. Return ? DefinePropertyOrThrow(object, propKey, desc). 
features: [generators]
---*/

assert.throws(TypeError, function() {
  class C { static *['prototype']() {} }
});
