// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of String conversion from Object value is conversion
    from primitive value
es5id: 9.8_A5_T1
description: Some objects convert to String by explicit transformation
---*/

// CHECK#1
if (String(new Number()) !== "0") {
  throw new Test262Error('#1: String(new Number()) === "0". Actual: ' + (String(new Number())));
}

// CHECK#2
if (String(new Number(0)) !== "0") {
  throw new Test262Error('#2: String(new Number(0)) === "0". Actual: ' + (String(new Number(0))));
}

// CHECK#3
if (String(new Number(Number.NaN)) !== "NaN") {
  throw new Test262Error('#3: String(new Number(Number.NaN)) === Not-a-Number. Actual: ' + (String(new Number(Number.NaN))));
}

// CHECK#4
if (String(new Number(null)) !== "0") {
  throw new Test262Error('#4: String(new Number(null)) === "0". Actual: ' + (String(new Number(null))));
}

// CHECK#5
if (String(new Number(void 0)) !== "NaN") {
  throw new Test262Error('#5: String(new Number(void 0)) === Not-a-Number. Actual: ' + (String(new Number(void 0))));
}

// CHECK#6
if (String(new Number(true)) !== "1") {
  throw new Test262Error('#6: String(new Number(true)) === "1". Actual: ' + (String(new Number(true))));
}

// CHECK#7
if (String(new Number(false)) !== "0") {
  throw new Test262Error('#7: String(new Number(false)) === "0". Actual: ' + (String(new Number(false))));
}

// CHECK#8
if (String(new Boolean(true)) !== "true") {
  throw new Test262Error('#8: String(new Boolean(true)) === "true". Actual: ' + (String(new Boolean(true))));
}

// CHECK#9
if (String(new Boolean(false)) !== "false") {
  throw new Test262Error('#9: Number(new Boolean(false)) === "false". Actual: ' + (Number(new Boolean(false))));
}

// CHECK#10
if (String(new Array(2, 4, 8, 16, 32)) !== "2,4,8,16,32") {
  throw new Test262Error('#10: String(new Array(2,4,8,16,32)) === "2,4,8,16,32". Actual: ' + (String(new Array(2, 4, 8, 16, 32))));
}

// CHECK#11
var myobj1 = {
  toNumber: function() {
    return 12345;
  },
  toString: function() {
    return 67890;
  },
  valueOf: function() {
    return "[object MyObj]";
  }
};

if (String(myobj1) !== "67890") {
  throw new Test262Error("#11: String(myobj) calls ToPrimitive with hint String");
}

// CHECK#12
var myobj2 = {
  toNumber: function() {
    return 12345;
  },
  toString: function() {
    return {}
  },
  valueOf: function() {
    return "[object MyObj]";
  }
};

if (String(myobj2) !== "[object MyObj]") {
  throw new Test262Error("#12: String(myobj) calls ToPrimitive with hint String");
}

// CHECK#13
var myobj3 = {
  toNumber: function() {
    return 12345;
  },
  valueOf: function() {
    return "[object MyObj]";
  }
};

if (String(myobj3) !== "[object Object]") {
  throw new Test262Error("#13: String(myobj) calls ToPrimitive with hint String");
}
