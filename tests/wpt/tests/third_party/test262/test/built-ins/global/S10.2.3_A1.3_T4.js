// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global object has properties such as built-in objects such as
    Math, String, Date, parseInt, etc
es5id: 10.2.3_A1.3_T4
description: Eval execution context - Other Properties
---*/

var evalStr =
'//CHECK#27\n'+
'if ( Math === null ) {\n'+
'  throw new Test262Error("#27: Math === null");\n'+
'}\n'+
';\n';

eval(evalStr);
