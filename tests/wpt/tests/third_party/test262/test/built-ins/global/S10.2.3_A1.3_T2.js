// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global object has properties such as built-in objects such as
    Math, String, Date, parseInt, etc
es5id: 10.2.3_A1.3_T2
description: Eval execution context - Function Properties
---*/

var evalStr =
'//CHECK#4\n'+
'if ( eval === null ) {\n'+
'  throw new Test262Error("#4: eval === null");\n'+
'}\n'+

'//CHECK#5\n'+
'if ( parseInt === null ) {\n'+
'  throw new Test262Error("#5: parseInt === null");\n'+
'}\n'+

'//CHECK#6\n'+
'if ( parseFloat === null ) {\n'+
'  throw new Test262Error("#6: parseFloat === null");\n'+
'}\n'+

'//CHECK#7\n'+
'if ( isNaN === null ) {\n'+
'  throw new Test262Error("#7: isNaN === null");\n'+
'}\n'+

'//CHECK#8\n'+
'if ( isFinite === null ) {\n'+
'  throw new Test262Error("#8: isFinite === null");\n'+
'}\n'+

'//CHECK#9\n'+
'if ( decodeURI === null ) {\n'+
'  throw new Test262Error("#9: decodeURI === null");\n'+
'}\n'+

'//CHECK#10\n'+
'if ( decodeURIComponent === null ) {\n'+
'  throw new Test262Error("#10: decodeURIComponent === null");\n'+
'}\n'+

'//CHECK#11\n'+
'if ( encodeURI === null ) {\n'+
'  throw new Test262Error("#11: encodeURI === null");\n'+
'}\n'+

'//CHECK#12\n'+
'if ( encodeURIComponent === null ) {\n'+
'  throw new Test262Error("#12: encodeURIComponent === null");\n'+
'}\n'+
';\n';

eval(evalStr);
