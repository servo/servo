// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Global object has properties such as built-in objects such as
    Math, String, Date, parseInt, etc
es5id: 10.2.3_A1.1_T3
description: Global execution context - Constructor Properties
---*/

//CHECK#13
if (Object === null) {
  throw new Test262Error("#13: Object === null");
}

//CHECK#14
if (Function === null) {
  throw new Test262Error("#14: Function === null");
}

//CHECK#15
if (String === null) {
  throw new Test262Error("#15: String === null");
}

//CHECK#16
if (Number === null) {
  throw new Test262Error("#16: Number === null");
}

//CHECK#17
if (Array === null) {
  throw new Test262Error("#17: Array === null");
}

//CHECK#18
if (Boolean === null) {
  throw new Test262Error("#20: Boolean === null");
}

//CHECK#18
if (Date === null) {
  throw new Test262Error("#18: Date === null");
}

//CHECK#19
if (RegExp === null) {
  throw new Test262Error("#19: RegExp === null");
}

//CHECK#20
if (Error === null) {
  throw new Test262Error("#20: Error === null");
}

//CHECK#21
if (EvalError === null) {
  throw new Test262Error("#21: EvalError === null");
}

//CHECK#22
if (RangeError === null) {
  throw new Test262Error("#22: RangeError === null");
}

//CHECK#23
if (ReferenceError === null) {
  throw new Test262Error("#23: ReferenceError === null");
}

//CHECK#24
if (SyntaxError === null) {
  throw new Test262Error("#24: SyntaxError === null");
}

//CHECK#25
if (TypeError === null) {
  throw new Test262Error("#25: TypeError === null");
}

//CHECK#26
if (URIError === null) {
  throw new Test262Error("#26: URIError === null");
}
