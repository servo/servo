// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString from array arguments
esid: sec-array.prototype.join
description: If Type(argument) is Object, evaluate ToPrimitive(argument, String)
---*/

var object = {
  valueOf: function() {
    return "+"
  }
};
var x = new Array(object);
assert.sameValue(x.join(), "[object Object]", 'x.join() must return "[object Object]"');

var object = {
  valueOf: function() {
    return "+"
  },
  toString: function() {
    return "*"
  }
};
var x = new Array(object);
assert.sameValue(x.join(), "*", 'x.join() must return "*"');

var object = {
  valueOf: function() {
    return "+"
  },
  toString: function() {
    return {}
  }
};
var x = new Array(object);
assert.sameValue(x.join(), "+", 'x.join() must return "+"');

try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return "*"
    }
  };
  var x = new Array(object);
  assert.sameValue(x.join(), "*", 'x.join() must return "*"');
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of e is not "error"');
}

var object = {
  toString: function() {
    return "*"
  }
};
var x = new Array(object);
assert.sameValue(x.join(), "*", 'x.join() must return "*"');

var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return "*"
  }
}
var x = new Array(object);
assert.sameValue(x.join(), "*", 'x.join() must return "*"');

try {
  var object = {
    valueOf: function() {
      return "+"
    },
    toString: function() {
      throw "error"
    }
  };
  var x = new Array(object);
  x.join();
  throw new Test262Error('#7.1: var object = {valueOf: function() {return "+"}, toString: function() {throw "error"}} var x = new Array(object); x.join() throw "error". Actual: ' + (x.join()));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}

try {
  var object = {
    valueOf: function() {
      return {}
    },
    toString: function() {
      return {}
    }
  };
  var x = new Array(object);
  x.join();
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}} var x = new Array(object); x.join() throw TypeError. Actual: ' + (x.join()));
}
catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}
