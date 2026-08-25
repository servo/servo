// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parseint-string-radix
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return 1
  }
};
assert.sameValue(parseInt(object), NaN, 'parseInt({valueOf: function() {return 1}}) must return NaN');

//CHECK#2
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return 0
  }
};

assert.sameValue(
  parseInt(object),
  0,
  'parseInt({valueOf: function() {return 1}, toString: function() {return 0}}) must return 0'
);

//CHECK#3
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return {}
  }
};

assert.sameValue(
  parseInt(object),
  1,
  'parseInt({valueOf: function() {return 1}, toString: function() {return {}}}) must return 1'
);

//CHECK#4
try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return 1
    }
  };

  assert.sameValue(
    parseInt(object),
    1,
    'parseInt({valueOf: function() {throw \\"error\\"}, toString: function() {return 1}}) must return 1'
  );
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of `e` is not "error"');
}

//CHECK#5
var object = {
  toString: function() {
    return 1
  }
};
assert.sameValue(parseInt(object), 1, 'parseInt({toString: function() {return 1}}) must return 1');

//CHECK#6
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return 1
  }
}

assert.sameValue(
  parseInt(object),
  1,
  'parseInt({valueOf: function() {return {}}, toString: function() {return 1}}) must return 1'
);

//CHECK#7
try {
  var object = {
    valueOf: function() {
      return 1
    },
    toString: function() {
      throw "error"
    }
  };
  parseInt(object);
  Test262Error.thrower('#7.1: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; parseInt(object) throw "error". Actual: ' + (parseInt(object)));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of `e` is "error"');
}

//CHECK#8
try {
  var object = {
    valueOf: function() {
      return {}
    },
    toString: function() {
      return {}
    }
  };
  parseInt(object);
  Test262Error.thrower('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; parseInt(object) throw TypeError. Actual: ' + (parseInt(object)));
}
catch (e) {
  assert.sameValue(e instanceof TypeError, true, 'The result of `(e instanceof TypeError)` is true');
}
