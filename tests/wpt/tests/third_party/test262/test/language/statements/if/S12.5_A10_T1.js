// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function expession inside the "if" expression is allowed
es5id: 12.5_A10_T1
description: >
    Using function expession(function __func(){return 0;}) inside the
    "if" expression
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#
if(function __func(){return 0;}){
    ;
}else {
    throw new Test262Error('#1: Function expession inside the "if" expression is allowed');
}
//
//////////////////////////////////////////////////////////////////////////////
