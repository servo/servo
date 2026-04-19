// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-super-keyword
es6id: 12.3.5
description: Binds the "this" value to value returned by "parent" constructor
info: |
  [...]
  6. Let result be ? Construct(func, argList, newTarget).
  7. Let thisER be GetThisEnvironment( ).
  8. Return ? thisER.BindThisValue(result).
features: [class]
---*/

var customThisValue = {};
var boundThisValue;
function Parent() {
  return customThisValue;
}

class Child extends Parent {
  constructor() {
    super();
    boundThisValue = this;
  }
}

new Child();

assert.sameValue(boundThisValue, customThisValue);
