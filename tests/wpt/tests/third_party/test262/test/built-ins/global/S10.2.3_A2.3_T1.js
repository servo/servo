// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.3_T1
description: Global execution context - Value Properties
---*/

var evalStr =
'//CHECK#1\n'+
'for (var x in this) {\n'+
'  if ( x === \'NaN\' ) {\n'+
'    throw new Test262Error("#1: \'NaN\' have attribute DontEnum");\n'+
'  } else if ( x === \'Infinity\' ) {\n'+
'    throw new Test262Error("#1: \'Infinity\' have attribute DontEnum");\n'+
'  } else if ( x === \'undefined\' ) {\n'+
'    throw new Test262Error("#1: \'undefined\' have attribute DontEnum");\n'+
'  }\n'+
'}\n';

eval(evalStr);
