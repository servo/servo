// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToNumber
esid: sec-parseint-string-radix
description: If Type(value) is Object, evaluate ToPrimitive(value, Number)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return 2
  }
};

assert.sameValue(
  parseInt("11", object),
  parseInt("11", 2),
  'parseInt("11", {valueOf: function() {return 2}}) must return the same value returned by parseInt("11", 2)'
);

//CHECK#2
var object = {
  valueOf: function() {
    return 2
  },
  toString: function() {
    return 1
  }
};

assert.sameValue(
  parseInt("11", object),
  parseInt("11", 2),
  'parseInt("11", {valueOf: function() {return 2}, toString: function() {return 1}}) must return the same value returned by parseInt("11", 2)'
);

//CHECK#3
var object = {
  valueOf: function() {
    return 2
  },
  toString: function() {
    return {}
  }
};

assert.sameValue(
  parseInt("11", object),
  parseInt("11", 2),
  'parseInt("11", {valueOf: function() {return 2}, toString: function() {return {}}}) must return the same value returned by parseInt("11", 2)'
);

//CHECK#4
try {
  var object = {
    valueOf: function() {
      return 2
    },
    toString: function() {
      throw "error"
    }
  };

  assert.sameValue(
    parseInt("11", object),
    parseInt("11", 2),
    'parseInt( "11", {valueOf: function() {return 2}, toString: function() {throw \\"error\\"}} ) must return the same value returned by parseInt("11", 2)'
  );
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of `e` is not "error"');
}

//CHECK#5
var object = {
  toString: function() {
    return 2
  }
};

assert.sameValue(
  parseInt("11", object),
  parseInt("11", 2),
  'parseInt("11", {toString: function() {return 2}}) must return the same value returned by parseInt("11", 2)'
);

//CHECK#6
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return 2
  }
}

assert.sameValue(
  parseInt("11", object),
  parseInt("11", 2),
  'parseInt("11", {valueOf: function() {return {}}, toString: function() {return 2}}) must return the same value returned by parseInt("11", 2)'
);

//CHECK#7
try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return 2
    }
  };
  parseInt("11", object);
  Test262Error.thrower('#7.1: var object = {valueOf: function() {throw "error"}, toString: function() {return 2}}; parseInt("11", object) throw "error". Actual: ' + (parseInt("11", object)));
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
  parseInt("11", object);
  Test262Error.thrower('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; parseInt("11", object) throw TypeError. Actual: ' + (parseInt("11", object)));
}
catch (e) {
  assert.sameValue(e instanceof TypeError, true, 'The result of `(e instanceof TypeError)` is true');
}
