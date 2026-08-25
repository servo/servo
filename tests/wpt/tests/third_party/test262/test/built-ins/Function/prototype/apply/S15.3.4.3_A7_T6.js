// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If argArray is either an array or an arguments object,
    the function is passed the (ToUint32(argArray.length)) arguments argArray[0], argArray[1],...,argArray[ToUint32(argArray.length)-1]
es5id: 15.3.4.3_A7_T6
description: argArray is (this, arguments), inside function declaration used
---*/

function FACTORY() {
  Function("a1,a2,a3", "this.shifted=a1+a2+a3;").apply(this, arguments);
}

var obj = new FACTORY("", 4, 2);

assert.sameValue(obj["shifted"], "42", 'The value of obj["shifted"] is expected to be "42"');

assert.sameValue(
  typeof this["shifted"],
  "undefined",
  'The value of `typeof this["shifted"]` is expected to be "undefined"'
);
