// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function argument that isn't provided has a value of undefined
es5id: 8.1_A5
description: Call function without provided argument
---*/

///////////////////////////////////////
//
function test(arg) {
// Check and make sure that arg is not undefined
	if (typeof(arg) !== "undefined") {
    throw new Test262Error('#1: Function argument that isn\'t provided has a value of undefined. Actual: ' + (typeof(arg)));
  }
}

test();
//
////////////////////////////////////////
