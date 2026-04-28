// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Function's scope chain is started when it is declared
es5id: 13.2.2_A19_T6
description: >
    Function is declared in the "object->do-while" scope, then the
    object is deleted and another object with the same name is declared
flags: [noStrict]
---*/

var a = 1;

var __obj = {a:2};

with (__obj)
{
    do {
        var __func = function()
        {
            return a;
        }
    } while(0);
}

delete __obj;

var __obj = {a:3};

with (__obj)
{
    result = __func();
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (result !== 2) {
	throw new Test262Error('#1: result === 2. Actual: result ==='+result);
}
//
//////////////////////////////////////////////////////////////////////////////
