// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If Type(Primitive(x)) is String or Type(Primitive(y)) is String, then
    operator x + y returns the result of concatenating ToString(x) followed
    by ToString(y)
es5id: 11.6.1_A3.2_T1.2
description: >
    Type(Primitive(x)) and Type(Primitive(y)) vary between Object
    object and Function object
---*/

//CHECK#1
if (({} + function(){return 1}) !== ({}.toString() + function(){return 1}.toString())) {
  throw new Test262Error('#1: ({} + function(){return 1}) === ({}.toString() + function(){return 1}.toString()). Actual: ' + (({} + function(){return 1})));
}

//CHECK#2
if ((function(){return 1} + {}) !== (function(){return 1}.toString() + {}.toString())) {
  throw new Test262Error('#2: (function(){return 1} + {}) === (function(){return 1}.toString() + {}.toString()). Actual: ' + ((function(){return 1} + {})));
}

//CHECK#3
if ((function(){return 1} + function(){return 1}) !== (function(){return 1}.toString() + function(){return 1}.toString())) {
  throw new Test262Error('#3: (function(){return 1} + function(){return 1}) === (function(){return 1}.toString() + function(){return 1}.toString()). Actual: ' + ((function(){return 1} + function(){return 1})));
}

//CHECK#4
if (({} + {}) !== ({}.toString() + {}.toString())) {
  throw new Test262Error('#4: ({} + {}) === ({}.toString() + {}.toString()). Actual: ' + (({} + {})));
}
