// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The global object does not have a [[Call]] property
es5id: 15.1_A2_T1
description: It is not possible to invoke the global object as a function
---*/

var global = this;

assert.throws(TypeError, function() {
  global();
});
