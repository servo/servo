// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"caller\" of bound function is poisoned (step 20)"
es5id: 15.3.4.5_A1
description: A bound function should fail to find its "caller"
---*/

function foo() {
  return bar.caller;
}
var bar = foo.bind({});

function baz() {
  return bar();
}

assert.throws(TypeError, function() {
  baz();
}, 'baz() throws a TypeError exception');
