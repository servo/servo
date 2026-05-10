// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Unicode symbols in function name are allowed
es5id: 13_A14
description: Defining function name with unicode symbols
---*/

var funcA = eval("function __func\u0041(__arg){return __arg;}; __funcA");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof funcA !== "function") {
    throw new Test262Error('#1: unicode symbols in function name are allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
