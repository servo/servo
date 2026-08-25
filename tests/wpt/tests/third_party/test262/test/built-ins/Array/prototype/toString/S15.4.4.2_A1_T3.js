// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tostring
info: |
    The result of calling this function is the same as if
    the built-in join method were invoked for this object with no argument
es5id: 15.4.4.2_A1_T3
description: Operator use ToString from array arguments
---*/

var x = new Array("", "", "");
if (x.toString() !== x.join()) {
  throw new Test262Error('#0.1: var x = new Array("","",""); x.toString() === x.join(). Actual: ' + (x.toString()));
} else {
  if (x.toString() !== ",,") {
    throw new Test262Error('#0.2: var x = new Array("","",""); x.toString() === ",,". Actual: ' + (x.toString()));
  }
}

var x = new Array("\\", "\\", "\\");
if (x.toString() !== x.join()) {
  throw new Test262Error('#1.1: var x = new Array("\\","\\","\\"); x.toString() === x.join(). Actual: ' + (x.toString()));
} else {
  if (x.toString() !== "\\,\\,\\") {
    throw new Test262Error('#1.2: var x = new Array("\\","\\","\\"); x.toString() === "\\,\\,\\". Actual: ' + (x.toString()));
  }
}

var x = new Array("&", "&", "&");
if (x.toString() !== x.join()) {
  throw new Test262Error('#2.1: var x = new Array("&", "&", "&"); x.toString() === x.join(). Actual: ' + (x.toString()));
} else {
  if (x.toString() !== "&,&,&") {
    throw new Test262Error('#2.2: var x = new Array("&", "&", "&"); x.toString() === "&,&,&". Actual: ' + (x.toString()));
  }
}

var x = new Array(true, true, true);
if (x.toString() !== x.join()) {
  throw new Test262Error('#3.1: var x = new Array(true,true,true); x.toString(true,true,true) === x.join(). Actual: ' + (x.toString(true, true, true)));
} else {
  if (x.toString() !== "true,true,true") {
    throw new Test262Error('#3.2: var x = new Array(true,true,true); x.toString(true,true,true) === "true,true,true". Actual: ' + (x.toString(true, true, true)));
  }
}

var x = new Array(null, null, null);
if (x.toString() !== x.join()) {
  throw new Test262Error('#4.1: var x = new Array(null,null,null); x.toString(null,null,null) === x.join(). Actual: ' + (x.toString(null, null, null)));
} else {
  if (x.toString() !== ",,") {
    throw new Test262Error('#4.2: var x = new Array(null,null,null); x.toString(null,null,null) === ",,". Actual: ' + (x.toString(null, null, null)));
  }
}

var x = new Array(undefined, undefined, undefined);
if (x.toString() !== x.join()) {
  throw new Test262Error('#5.1: var x = new Array(undefined,undefined,undefined); x.toString(undefined,undefined,undefined) === x.join(). Actual: ' + (x.toString(undefined, undefined, undefined)));
} else {
  if (x.toString() !== ",,") {
    throw new Test262Error('#5.2: var x = new Array(undefined,undefined,undefined); x.toString(undefined,undefined,undefined) === ",,". Actual: ' + (x.toString(undefined, undefined, undefined)));
  }
}

var x = new Array(Infinity, Infinity, Infinity);
if (x.toString() !== x.join()) {
  throw new Test262Error('#6.1: var x = new Array(Infinity,Infinity,Infinity); x.toString(Infinity,Infinity,Infinity) === x.join(). Actual: ' + (x.toString(Infinity, Infinity, Infinity)));
} else {
  if (x.toString() !== "Infinity,Infinity,Infinity") {
    throw new Test262Error('#6.2: var x = new Array(Infinity,Infinity,Infinity); x.toString(Infinity,Infinity,Infinity) === "Infinity,Infinity,Infinity". Actual: ' + (x.toString(Infinity, Infinity, Infinity)));
  }
}

var x = new Array(NaN, NaN, NaN);
if (x.toString() !== x.join()) {
  throw new Test262Error('#7.1: var x = new Array(NaN,NaN,NaN); x.toString(NaN,NaN,NaN) === x.join(). Actual: ' + (x.toString(NaN, NaN, NaN)));
} else {
  if (x.toString() !== "NaN,NaN,NaN") {
    throw new Test262Error('#7.2: var x = new Array(NaN,NaN,NaN); x.toString(NaN,NaN,NaN) === "NaN,NaN,NaN". Actual: ' + (x.toString(NaN, NaN, NaN)));
  }
}
