// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-ecmascript-function-objects-call-thisargument-argumentslist
description: Error when invoking a class constructor
info: |
  [...]
  2. If F's [[FunctionKind]] internal slot is "classConstructor", throw a
     TypeError exception.
features: [class]
---*/

class C {}

assert.throws(TypeError, function() {
  C();
});
