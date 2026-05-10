// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Result of number conversion from object value is the result
    of conversion from primitive value
es5id: 9.3_A5_T1
description: >
    new Number(), new Number(0), new Number(Number.NaN), new
    Number(null),  new Number(void 0) and others convert to Number by
    explicit transformation
---*/
assert.sameValue(Number(new Number()), 0, 'Number(new Number()) must return 0');
assert.sameValue(Number(new Number(0)), 0, 'Number(new Number(0)) must return 0');

// CHECK#3
assert.sameValue(Number(new Number(NaN)), NaN, 'Number(new Number(NaN)) returns NaN');

assert.sameValue(Number(new Number(null)), 0, 'Number(new Number(null)) must return 0');

// CHECK#5
assert.sameValue(Number(new Number(void 0)), NaN, 'Number(new Number(void 0)) returns NaN');

assert.sameValue(Number(new Number(true)), 1, 'Number(new Number(true)) must return 1');
assert.sameValue(Number(new Number(false)), +0, 'Number(new Number(false)) must return +0');
assert.sameValue(Number(new Boolean(true)), 1, 'Number(new Boolean(true)) must return 1');
assert.sameValue(Number(new Boolean(false)), +0, 'Number(new Boolean(false)) must return +0');

// CHECK#10
assert.sameValue(Number(new Array(2, 4, 8, 16, 32)), NaN, 'Number(new Array(2, 4, 8, 16, 32)) returns NaN');

// CHECK#11
var myobj1 = {
  ToNumber: function() {
    return 12345;
  },
  toString: function() {
    return "67890";
  },
  valueOf: function() {
    return "[object MyObj]";
  }
};

assert.sameValue(Number(myobj1), NaN, 'Number("{ToNumber: function() {return 12345;}, toString: function() {return "67890";}, valueOf: function() {return "[object MyObj]";}}) returns NaN');

// CHECK#12
var myobj2 = {
  ToNumber: function() {
    return 12345;
  },
  toString: function() {
    return "67890";
  },
  valueOf: function() {
    return "9876543210";
  }
};

assert.sameValue(
  Number(myobj2),
  9876543210,
  'Number("{ToNumber: function() {return 12345;}, toString: function() {return "67890";}, valueOf: function() {return "9876543210";}}) must return 9876543210'
);


// CHECK#13
var myobj3 = {
  ToNumber: function() {
    return 12345;
  },
  toString: function() {
    return "[object MyObj]";
  }
};

assert.sameValue(Number(myobj3), NaN, 'Number("{ToNumber: function() {return 12345;}, toString: function() {return "[object MyObj]";}}) returns NaN');

// CHECK#14
var myobj4 = {
  ToNumber: function() {
    return 12345;
  },
  toString: function() {
    return "67890";
  }
};

assert.sameValue(
  Number(myobj4),
  67890,
  'Number("{ToNumber: function() {return 12345;}, toString: function() {return "67890";}}) must return 67890'
);

// CHECK#15
var myobj5 = {
  ToNumber: function() {
    return 12345;
  }
};

assert.sameValue(Number(myobj5), NaN, 'Number({ToNumber: function() {return 12345;}}) returns NaN');
