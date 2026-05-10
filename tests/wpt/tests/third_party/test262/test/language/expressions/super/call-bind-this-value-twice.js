// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Abrupt completion from re-binding "this" value
info: |
  [...]
  6. Let result be ? Construct(func, argList, newTarget).
  7. Let thisER be GetThisEnvironment( ).
  8. Return ? thisER.BindThisValue(result).

  8.1.1.3.1 BindThisValue

  1. Let envRec be the function Environment Record for which the method was
     invoked.
  2. Assert: envRec.[[ThisBindingStatus]] is not "lexical".
  3. If envRec.[[ThisBindingStatus]] is "initialized", throw a ReferenceError
     exception.
features: [class]
---*/

var caught;
function Parent() {}

class Child extends Parent {
  constructor() {
    super();
    try {
      super();
    } catch (err) {
      caught = err;
    }
  }
}

new Child();

assert.sameValue(typeof caught, 'object');
assert.sameValue(caught.constructor, ReferenceError);
