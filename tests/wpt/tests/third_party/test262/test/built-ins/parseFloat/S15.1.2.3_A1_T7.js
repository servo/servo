// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-parsefloat-string
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return 1
  }
};
assert.sameValue(parseFloat(object), NaN, "{valueOf: function() {return 1}}");

//CHECK#2
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return 0
  }
};
if (parseFloat(object) !== 0) {
  throw new Test262Error('#2: var object = {valueOf: function() {return 1}, toString: function() {return 0}}; parseFloat(object) === 0. Actual: ' + (parseFloat(object)));
}

//CHECK#3
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return {}
  }
};
if (parseFloat(object) !== 1) {
  throw new Test262Error('#3: var object = {valueOf: function() {return 1}, toString: function() {return {}}}; parseFloat(object) === 1. Actual: ' + (parseFloat(object)));
}

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
  if (parseFloat(object) !== 1) {
    throw new Test262Error('#4.1: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; parseFloat(object) === 1. Actual: ' + (parseFloat(object)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; parseFloat(object) not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; parseFloat(object) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return 1
  }
};
if (parseFloat(object) !== 1) {
  throw new Test262Error('#5: var object = {toString: function() {return 1}}; parseFloat(object) === 1. Actual: ' + (parseFloat(object)));
}

//CHECK#6
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return 1
  }
}
if (parseFloat(object) !== 1) {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return 1}}; parseFloat(object) === 1. Actual: ' + (parseFloat(object)));
}

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
  parseFloat(object);
  throw new Test262Error('#7.1: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; parseFloat(object) throw "error". Actual: ' + (parseFloat(object)));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; parseFloat(object) throw "error". Actual: ' + (e));
  }
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
  parseFloat(object);
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; parseFloat(object) throw TypeError. Actual: ' + (parseFloat(object)));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; parseFloat(object) throw TypeError. Actual: ' + (e));
  }
}
