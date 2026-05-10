// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global object has properties such as built-in objects such as
    Math, String, Date, parseInt, etc
es5id: 10.2.3_A1.3_T3
description: Eval execution context - Constructor Properties
---*/

var evalStr =
'//CHECK#13\n'+
'if ( Object === null ) {\n'+
'  throw new Test262Error("#13: Object === null");\n'+
'}\n'+

'//CHECK#14\n'+
'if ( Function === null ) {\n'+
'  throw new Test262Error("#14: Function === null");\n'+
'}\n'+

'//CHECK#15\n'+
'if ( String === null ) {\n'+
'  throw new Test262Error("#15: String === null");\n'+
'}\n'+

'//CHECK#16\n'+
'if ( Number === null ) {\n'+
'  throw new Test262Error("#16: Function === null");\n'+
'}\n'+

'//CHECK#17\n'+
'if ( Array === null ) {\n'+
'  throw new Test262Error("#17: Array === null");\n'+
'}\n'+

'//CHECK#18\n'+
'if ( Boolean === null ) {\n'+
'  throw new Test262Error("#20: Boolean === null");\n'+
'}\n'+

'//CHECK#18\n'+
'if ( Date === null ) {\n'+
'  throw new Test262Error("#18: Date === null");\n'+
'}\n'+

'//CHECK#19\n'+
'if ( RegExp === null ) {\n'+
'  throw new Test262Error("#19: RegExp === null");\n'+
'}\n'+

'//CHECK#20\n'+
'if ( Error === null ) {\n'+
'  throw new Test262Error("#20: Error === null");\n'+
'}\n'+

'//CHECK#21\n'+
'if ( EvalError === null ) {\n'+
'  throw new Test262Error("#21: EvalError === null");\n'+
'}\n'+

'//CHECK#22\n'+
'if ( RangeError === null ) {\n'+
'  throw new Test262Error("#22: RangeError === null");\n'+
'}\n'+

'//CHECK#23\n'+
'if ( ReferenceError === null ) {\n'+
'  throw new Test262Error("#23: ReferenceError === null");\n'+
'}\n'+

'//CHECK#24\n'+
'if ( SyntaxError === null ) {\n'+
'  throw new Test262Error("#24: SyntaxError === null");\n'+
'}\n'+

'//CHECK#25\n'+
'if ( TypeError === null ) {\n'+
'  throw new Test262Error("#25: TypeError === null");\n'+
'}\n'+

'//CHECK#26\n'+
'if ( URIError === null ) {\n'+
'  throw new Test262Error("#26: URIError === null");\n'+
'}\n'+
';\n';

eval(evalStr);
