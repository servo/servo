// Copyright (C) 2018 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-makesuperpropertyreference
description: >
  SuperProperty evaluation order: super() thisBinding initialization must occur first.
---*/
class Derived extends Object {
  constructor() {
    super[super()];
    throw new Test262Error();
  }
}

assert.throws(ReferenceError, function() {
  new Derived();
}, '`super[super()]` via `new Derived()` throws a ReferenceError');
