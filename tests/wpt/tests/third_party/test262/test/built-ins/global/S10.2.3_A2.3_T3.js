// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.3_T3
description: Global execution context - Constructor Properties
---*/

var evalStr =
'//CHECK#1\n'+
'for (var x in this) {\n'+
'  if ( x === \'Object\' ) {\n'+
'    throw new Test262Error("#1: \'Object\' have attribute DontEnum");\n'+
'  } else if ( x === \'Function\') {\n'+
'    throw new Test262Error("#1: \'Function\' have attribute DontEnum");\n'+
'  } else if ( x === \'String\' ) {\n'+
'    throw new Test262Error("#1: \'String\' have attribute DontEnum");\n'+
'  } else if ( x === \'Number\' ) {\n'+
'    throw new Test262Error("#1: \'Number\' have attribute DontEnum");\n'+
'  } else if ( x === \'Array\' ) {\n'+
'    throw new Test262Error("#1: \'Array\' have attribute DontEnum");\n'+
'  } else if ( x === \'Boolean\' ) {\n'+
'    throw new Test262Error("#1: \'Boolean\' have attribute DontEnum");\n'+
'  } else if ( x === \'Date\' ) {\n'+
'    throw new Test262Error("#1: \'Date\' have attribute DontEnum");\n'+
'  } else if ( x === \'RegExp\' ) {\n'+
'    throw new Test262Error("#1: \'RegExp\' have attribute DontEnum");\n'+
'  } else if ( x === \'Error\' ) {\n'+
'    throw new Test262Error("#1: \'Error\' have attribute DontEnum");\n'+
'  } else if ( x === \'EvalError\' ) {\n'+
'    throw new Test262Error("#1: \'EvalError\' have attribute DontEnum");\n'+
'  } else if ( x === \'RangeError\' ) {\n'+
'    throw new Test262Error("#1: \'RangeError\' have attribute DontEnum");\n'+
'  } else if ( x === \'ReferenceError\' ) {\n'+
'    throw new Test262Error("#1: \'ReferenceError\' have attribute DontEnum");\n'+
'  } else if ( x === \'SyntaxError\' ) {\n'+
'    throw new Test262Error("#1: \'SyntaxError\' have attribute DontEnum");\n'+
'  } else if ( x === \'TypeError\' ) {\n'+
'    throw new Test262Error("#1: \'TypeError\' have attribute DontEnum");\n'+
'  } else if ( x === \'URIError\' ) {\n'+
'    throw new Test262Error("#1: \'URIError\' have attribute DontEnum");\n'+
'  }\n'+
'}\n';

eval(evalStr);
