// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.3_T2
description: Global execution context - Function Properties
---*/

var evalStr =
'//CHECK#1\n'+
'for (var x in this) {\n'+
'  if ( x === \'eval\' ) {\n'+
'    throw new Test262Error("#1: \'eval\' have attribute DontEnum");\n'+
'  } else if ( x === \'parseInt\' ) {\n'+
'    throw new Test262Error("#1: \'parseInt\' have attribute DontEnum");\n'+
'  } else if ( x === \'parseFloat\' ) {\n'+
'    throw new Test262Error("#1: \'parseFloat\' have attribute DontEnum");\n'+
'  } else if ( x === \'isNaN\' ) {\n'+
'    throw new Test262Error("#1: \'isNaN\' have attribute DontEnum");\n'+
'  } else if ( x === \'isFinite\' ) {\n'+
'    throw new Test262Error("#1: \'isFinite\' have attribute DontEnum");\n'+
'  } else if ( x === \'decodeURI\' ) {\n'+
'    throw new Test262Error("#1: \'decodeURI\' have attribute DontEnum");\n'+
'  } else if ( x === \'decodeURIComponent\' ) {\n'+
'    throw new Test262Error("#1: \'decodeURIComponent\' have attribute DontEnum");\n'+
'  } else if ( x === \'encodeURI\' ) {\n'+
'    throw new Test262Error("#1: \'encodeURI\' have attribute DontEnum");\n'+
'  } else if ( x === \'encodeURIComponent\' ) {\n'+
'    throw new Test262Error("#1: \'encodeURIComponent\' have attribute DontEnum");\n'+
'  }\n'+
'}\n';

eval(evalStr);
