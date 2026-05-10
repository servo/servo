// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.6-6-1
description: "'length property of arguments object exists"
---*/

function testcase() {
  var desc = Object.getOwnPropertyDescriptor(arguments,"length");
  assert.notSameValue(desc, undefined);
}
testcase();
