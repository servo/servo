// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The join function is intentionally generic.
    It does not require that its this value be an Array object
esid: sec-array.prototype.join
description: >
    Operator use ToNumber from length.  If Type(value) is Object,
    evaluate ToPrimitive(value, Number)
---*/

var obj = {};
obj.join = Array.prototype.join;

obj.length = {
  valueOf() {
    return 3
  }
};
assert.sameValue(obj.join(), ",,", 'obj.join() must return ",,"');

obj.length = {
  valueOf() {
    return 3
  },
  toString() {
    return 2
  }
};
assert.sameValue(obj.join(), ",,", 'obj.join() must return ",,"');

obj.length = {
  valueOf() {
    return 3
  },
  toString() {
    return {}
  }
};
assert.sameValue(obj.join(), ",,", 'obj.join() must return ",,"');

obj.length = {
  valueOf() {
    return 3
  },
  toString() {
    throw new Test262Error();
  }
};
assert.sameValue(obj.join(), ",,", 'obj.join() must return ",,"');

obj.length = {
  toString() {
    return 2
  }
};
assert.sameValue(obj.join(), ",", 'obj.join() must return ","');

obj.length = {
  valueOf() {
    return {}
  },
  toString() {
    return 2
  }
}
assert.sameValue(obj.join(), ",", 'obj.join() must return ","');

assert.throws(Test262Error, () => {
  obj.length = {
    valueOf() {
      throw new Test262Error();
    },
    toString() {
      return 2
    }
  };
  obj.join();
});

assert.throws(TypeError, () => {
  obj.length = {
    valueOf() {
      return {}
    },
    toString() {
      return {}
    }
  };
  obj.join();
});
