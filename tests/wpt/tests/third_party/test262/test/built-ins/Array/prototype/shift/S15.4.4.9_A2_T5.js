// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The shift function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.shift
description: >
    Operator use ToNumber from length.  If Type(value) is Object,
    evaluate ToPrimitive(value, Number)
---*/

var obj = {};
obj.shift = Array.prototype.shift;

obj[0] = -1;
obj.length = {
  valueOf() {
    return 1
  }
};
var shift = obj.shift();
assert.sameValue(shift, -1, 'The value of shift is expected to be -1');

obj[0] = -1;
obj.length = {
  valueOf() {
    return 1
  },
  toString() {
    return 0
  }
};
var shift = obj.shift();
assert.sameValue(shift, -1, 'The value of shift is expected to be -1');

obj[0] = -1;
obj.length = {
  valueOf() {
    return 1
  },
  toString() {
    return {}
  }
};
var shift = obj.shift();
assert.sameValue(shift, -1, 'The value of shift is expected to be -1');

try {
  obj[0] = -1;
  obj.length = {
    valueOf() {
      return 1
    },
    toString() {
      throw "error"
    }
  };
  var shift = obj.shift();
  assert.sameValue(shift, -1, 'The value of shift is expected to be -1');
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of e is not "error"');
}

obj[0] = -1;
obj.length = {
  toString() {
    return 0
  }
};
var shift = obj.shift();
assert.sameValue(shift, undefined, 'The value of shift is expected to equal undefined');

obj[0] = -1;
obj.length = {
  valueOf() {
    return {}
  },
  toString() {
    return 0
  }
}
var shift = obj.shift();
assert.sameValue(shift, undefined, 'The value of shift is expected to equal undefined');

try {
  obj[0] = -1;
  obj.length = {
    valueOf() {
      throw "error"
    },
    toString() {
      return 0
    }
  };
  var shift = obj.shift();
  throw new Test262Error('#7.1: obj[0] = -1; obj.length = {valueOf() {throw "error"}, toString() {return 0}}; obj.shift() throw "error". Actual: ' + (shift));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}

try {
  obj[0] = -1;
  obj.length = {
    valueOf() {
      return {}
    },
    toString() {
      return {}
    }
  };
  var shift = obj.shift();
  throw new Test262Error('#8.1: obj[0] = -1; obj.length = {valueOf() {return {}}, toString() {return {}}}  obj.shift() throw TypeError. Actual: ' + (shift));
}
catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}
