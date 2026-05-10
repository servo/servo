// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"arguments\" of bound function is poisoned (step 21)"
es5id: 15.3.4.5_A2
description: a bound function should fail to find the bound function "arguments"
---*/

function foo() {
  return bar.arguments;
}
var bar = foo.bind({});

function baz() {
  return bar();
}

assert.throws(TypeError, function() {
  baz();
}, 'baz() throws a TypeError exception');
