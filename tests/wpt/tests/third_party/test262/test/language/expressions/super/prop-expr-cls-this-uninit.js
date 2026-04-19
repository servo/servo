// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: >
  SuperProperty evaluation when "this" binding has not been initialized
info: |
  [...]
  4. If the code matched by the syntactic production that is being evaluated is
     strict mode code, let strict be true, else let strict be false.
  5. Return ? MakeSuperPropertyReference(propertyKey, strict).

  12.3.5.3 Runtime Semantics: MakeSuperPropertyReference

  1. Let env be GetThisEnvironment( ).
  2. If env.HasSuperBinding() is false, throw a ReferenceError exception.
  3. Let actualThis be ? env.GetThisBinding().

  8.1.1.3.4 GetThisBinding

  1. Let envRec be the function Environment Record for which the method was
     invoked.
  2. Assert: envRec.[[ThisBindingStatus]] is not "lexical".
  3. If envRec.[[ThisBindingStatus]] is "uninitialized", throw a ReferenceError
     exception.
features: [class]
---*/

var caught;
class C extends Object {
  constructor() {
    try {
      super['x'];
    } catch (err) {
      caught = err;
    }
  }
}

// When the "construct" invocation completes and the "this" value is
// uninitialized, the specification dictates that a ReferenceError must be
// thrown. That behavior is tested elsewhere, so the error is ignored (if it is
// produced at all).
try {
  new C();
} catch (_) {}

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, ReferenceError);
