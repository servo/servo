// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The push function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.push
description: >
    Operator use ToNumber from length.  If Type(value) is Object,
    evaluate ToPrimitive(value, Number)
---*/

var obj = {};
obj.push = Array.prototype.push;

obj.length = {
  valueOf() {
    return 3
  }
};
var push = obj.push();
assert.sameValue(push, 3, 'The value of push is expected to be 3');

obj.length = {
  valueOf() {
    return 3
  },
  toString() {
    return 1
  }
};
var push = obj.push();
assert.sameValue(push, 3, 'The value of push is expected to be 3');

obj.length = {
  valueOf() {
    return 3
  },
  toString() {
    return {}
  }
};
var push = obj.push();
assert.sameValue(push, 3, 'The value of push is expected to be 3');

try {
  obj.length = {
    valueOf() {
      return 3
    },
    toString() {
      throw "error"
    }
  };
  var push = obj.push();
  assert.sameValue(push, 3, 'The value of push is expected to be 3');
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of e is not "error"');
}

obj.length = {
  toString() {
    return 1
  }
};
var push = obj.push();
assert.sameValue(push, 1, 'The value of push is expected to be 1');

obj.length = {
  valueOf() {
    return {}
  },
  toString() {
    return 1
  }
}
var push = obj.push();
assert.sameValue(push, 1, 'The value of push is expected to be 1');

try {

  obj.length = {
    valueOf() {
      throw "error"
    },
    toString() {
      return 1
    }
  };
  var push = obj.push();
  throw new Test262Error('#7.1:  obj.length = {valueOf() {throw "error"}, toString() {return 1}}; obj.push() throw "error". Actual: ' + (push));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}

try {

  obj.length = {
    valueOf() {
      return {}
    },
    toString() {
      return {}
    }
  };
  var push = obj.push();
  throw new Test262Error('#8.1:  obj.length = {valueOf() {return {}}, toString() {return {}}}  obj.push() throw TypeError. Actual: ' + (push));
}
catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}
