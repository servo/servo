// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.tostring
info: |
    The result of calling this function is the same as if
    the built-in join method were invoked for this object with no argument
es5id: 15.4.4.2_A1_T4
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

var object = {
  valueOf() {
    return "+"
  }
};
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');

var object = {
  valueOf() {
    return "+"
  },
  toString() {
    return "*"
  }
};
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');

var object = {
  valueOf() {
    return "+"
  },
  toString() {
    return {}
  }
};
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');

var object = {
  valueOf() {
    throw "error"
  },
  toString() {
    return "*"
  }
};
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');


var object = {
  toString() {
    return "*"
  }
};
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');

var object = {
  valueOf() {
    return {}
  },
  toString() {
    return "*"
  }
}
var x = new Array(object);
assert.sameValue(x.toString(), x.join(), 'x.toString() must return the same value returned by x.join()');

assert.throws(Test262Error, () => {
  var object = {
    valueOf() {
      return "+"
    },
    toString() {
      throw new Test262Error();
    }
  };
  var x = new Array(object);
  x.toString();
});

assert.throws(TypeError, () => {
  var object = {
    valueOf() {
      return {}
    },
    toString() {
      return {}
    }
  };
  var x = new Array(object);
  x.toString();
});
