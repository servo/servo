// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-5-1
description: >
    [[Prototype]] property of Arguments is set to Object prototype
    object
---*/

function testcase() {
  assert.sameValue(Object.getPrototypeOf(arguments), Object.getPrototypeOf({}));
}
testcase();
