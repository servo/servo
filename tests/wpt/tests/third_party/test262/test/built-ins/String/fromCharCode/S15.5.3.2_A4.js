// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.fromCharCode has not [[construct]] method
es5id: 15.5.3.2_A4
description: Checking if creating "new String.fromCharCode" fails
---*/

var __fcc__func = String.fromCharCode;

delete String.fromCharCode;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var __obj = new __fcc__func(65, 66, 66, 65);
  throw new Test262Error('#1: __fcc__func = String.fromCharCode; var __obj = new __fcc__func(65,66,66,65) lead to throwing exception');
} catch (e) {
  if (e instanceof Test262Error) throw e;
}
//
//////////////////////////////////////////////////////////////////////////////
