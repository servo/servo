// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The PropertyName is undefined, ToString(BooleanLiteral),
    ToString(nullLiteral)
es5id: 11.1.5_A4.3
description: "Creating properties with following names: undefined, 'true', 'null'"
---*/

//CHECK#1
var object = {undefined : true};
if (object.undefined !== true) {
  throw new Test262Error('#1: var object = {undefined : true}; object.undefined === true');
}

//CHECK#2
var object = {undefined : true};
if (object["undefined"] !== true) {
  throw new Test262Error('#2: var object = {undefined : true}; object["undefined"] === true');
}

//CHECK#3
var object = {"true" : true};
if (object["true"] !== true) {
  throw new Test262Error('#3: var object = {"true" : true}; object["true"] === true');
}

//CHECK#4
var object = {"null" : true};
if (object["null"] !== true) {
  throw new Test262Error('#4: var object = {"null" : true}; object["null"] === true');
}
