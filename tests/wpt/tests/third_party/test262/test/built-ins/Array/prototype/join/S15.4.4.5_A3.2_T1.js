// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString from array arguments
esid: sec-array.prototype.join
description: >
    Checking arguments and separator in ["", "\\", "&", true,
    Infinity, null, undefind, NaN]
---*/

var x = new Array("", "", "");
if (x.join("") !== "") {
  throw new Test262Error('#0: var x = new Array("","",""); x.join("") === "". Actual: ' + (x.join("")));
}

var x = new Array("\\", "\\", "\\");
if (x.join("\\") !== "\\\\\\\\\\") {
  throw new Test262Error('#1: var x = new Array("\\","\\","\\"); x.join("\\") === "\\\\\\\\\\". Actual: ' + (x.join("\\")));
}

var x = new Array("&", "&", "&");
if (x.join("&") !== "&&&&&") {
  throw new Test262Error('#2: var x = new Array("&", "&", "&"); x.join("&") === "&&&&&". Actual: ' + (x.join("&")));
}

var x = new Array(true, true, true);
if (x.join() !== "true,true,true") {
  throw new Test262Error('#3: var x = new Array(true,true,true); x.join(true,true,true) === "true,true,true". Actual: ' + (x.join(true, true, true)));
}

var x = new Array(null, null, null);
if (x.join() !== ",,") {
  throw new Test262Error('#4: var x = new Array(null,null,null); x.join(null,null,null) === ",,". Actual: ' + (x.join(null, null, null)));
}

var x = new Array(undefined, undefined, undefined);
if (x.join() !== ",,") {
  throw new Test262Error('#5: var x = new Array(undefined,undefined,undefined); x.join(undefined,undefined,undefined) === ",,". Actual: ' + (x.join(undefined, undefined, undefined)));
}

var x = new Array(Infinity, Infinity, Infinity);
if (x.join() !== "Infinity,Infinity,Infinity") {
  throw new Test262Error('#6: var x = new Array(Infinity,Infinity,Infinity); x.join(Infinity,Infinity,Infinity) === "Infinity,Infinity,Infinity". Actual: ' + (x.join(Infinity, Infinity, Infinity)));
}

var x = new Array(NaN, NaN, NaN);
if (x.join() !== "NaN,NaN,NaN") {
  throw new Test262Error('#7: var x = new Array(NaN,NaN,NaN); x.join(NaN,NaN,NaN) === "NaN,NaN,NaN". Actual: ' + (x.join(NaN, NaN, NaN)));
}
