// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property name P (in the form of a string value) is an array index
    if and only if ToString(ToUint32(P)) is equal to P and ToUint32(P) is not equal to 2^32 - 1
es5id: 15.4_A1.1_T9
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

var x = [];
var object = {
  valueOf: function() {
    return 1
  }
};
x[object] = 0;
assert.sameValue(x["[object Object]"], 0, 'The value of x["[object Object]"] is expected to be 0');

x = [];
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return 0
  }
};
x[object] = 0;
assert.sameValue(x[0], 0, 'The value of x[0] is expected to be 0');

x = [];
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return {}
  }
};
x[object] = 0;
assert.sameValue(x[1], 0, 'The value of x[1] is expected to be 0');

try {
  x = [];
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return 1
    }
  };
  x[object] = 0;
  assert.sameValue(x[1], 0, 'The value of x[1] is expected to be 0');
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of e is not "error"');
}

x = [];
var object = {
  toString: function() {
    return 1
  }
};
x[object] = 0;
assert.sameValue(x[1], 0, 'The value of x[1] is expected to be 0');

x = [];
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return 1
  }
}
x[object] = 0;
assert.sameValue(x[1], 0, 'The value of x[1] is expected to be 0');

try {
  x = [];
  var object = {
    valueOf: function() {
      return 1
    },
    toString: function() {
      throw "error"
    }
  };
  x[object];
  throw new Test262Error('#7.1: x = []; var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; x[object] throw "error". Actual: ' + (x[object]));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}

try {
  x = [];
  var object = {
    valueOf: function() {
      return {}
    },
    toString: function() {
      return {}
    }
  };
  x[object];
  throw new Test262Error('#8.1: x = []; var object = {valueOf: function() {return {}}, toString: function() {return {}}}; x[object] throw TypeError. Actual: ' + (x[object]));
}
catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}
