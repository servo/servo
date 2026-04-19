// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.11.4.4-8-2
description: >
    Error.prototype.toString return empty string when 'name' is empty
    string and 'msg' is undefined
---*/

var errObj = new Error();
errObj.name = "";
if (errObj.name !== "") {
  throw new Test262Error("Expected errObj.name to be '', actually " + errObj.name);
}
if (errObj.toString() !== "") {
  throw new Test262Error("Expected errObj.toString() to be '', actually " + errObj.toString());
}
