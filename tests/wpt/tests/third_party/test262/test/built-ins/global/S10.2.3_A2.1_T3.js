// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Global object properties have attributes { DontEnum }
es5id: 10.2.3_A2.1_T3
description: Global execution context - Constructor Properties
---*/

//CHECK#1
for (var x in this) {
  if (x === 'Object') {
    throw new Test262Error("#1: 'property 'Object' have attribute DontEnum");
  } else if (x === 'Function') {
    throw new Test262Error("#1: 'Function' have attribute DontEnum");
  } else if (x === 'String') {
    throw new Test262Error("#1: 'String' have attribute DontEnum");
  } else if (x === 'Number') {
    throw new Test262Error("#1: 'Number' have attribute DontEnum");
  } else if (x === 'Array') {
    throw new Test262Error("#1: 'Array' have attribute DontEnum");
  } else if (x === 'Boolean') {
    throw new Test262Error("#1: 'Boolean' have attribute DontEnum");
  } else if (x === 'Date') {
    throw new Test262Error("#1: 'Date' have attribute DontEnum");
  } else if (x === 'RegExp') {
    throw new Test262Error("#1: 'RegExp' have attribute DontEnum");
  } else if (x === 'Error') {
    throw new Test262Error("#1: 'Error' have attribute DontEnum");
  } else if (x === 'EvalError') {
    throw new Test262Error("#1: 'EvalError' have attribute DontEnum");
  } else if (x === 'RangeError') {
    throw new Test262Error("#1: 'RangeError' have attribute DontEnum");
  } else if (x === 'ReferenceError') {
    throw new Test262Error("#1: 'ReferenceError' have attribute DontEnum");
  } else if (x === 'SyntaxError') {
    throw new Test262Error("#1: 'SyntaxError' have attribute DontEnum");
  } else if (x === 'TypeError') {
    throw new Test262Error("#1: 'TypeError' have attribute DontEnum");
  } else if (x === 'URIError') {
    throw new Test262Error("#1: 'URIError' have attribute DontEnum");
  }
}
