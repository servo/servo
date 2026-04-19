// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-decodeuri-encodeduri
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return "%5E"
  }
};
if (decodeURI(object) !== "[object Object]") {
  throw new Test262Error('#1: var object = {valueOf: function() {return "%5E"}}; decodeURI(object) === [object Object]. Actual: ' + (decodeURI(object)));
}

//CHECK#2
var object = {
  valueOf: function() {
    return ""
  },
  toString: function() {
    return "%5E"
  }
};
if (decodeURI(object) !== "^") {
  throw new Test262Error('#2: var object = {valueOf: function() {return ""}, toString: function() {return "%5E"}}; decodeURI(object) === "^". Actual: ' + (decodeURI(object)));
}

//CHECK#3
var object = {
  valueOf: function() {
    return "%5E"
  },
  toString: function() {
    return {}
  }
};
if (decodeURI(object) !== "^") {
  throw new Test262Error('#3: var object = {valueOf: function() {return "%5E"}, toString: function() {return {}}}; decodeURI(object) === "^". Actual: ' + (decodeURI(object)));
}

//CHECK#4
try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return "%5E"
    }
  };
  if (decodeURI(object) !== "^") {
    throw new Test262Error('#4.1: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURI(object) === "^". Actual: ' + (decodeURI(object)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURI(object) not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURI(object) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return "%5E"
  }
};
if (decodeURI(object) !== "^") {
  throw new Test262Error('#5: var object = {toString: function() {return "%5E"}}; decodeURI(object) === "^". Actual: ' + (decodeURI(object)));
}

//CHECK#6
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return "%5E"
  }
}
if (decodeURI(object) !== "^") {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return "%5E"}}; decodeURI(object) === "^". Actual: ' + (decodeURI(object)));
}

//CHECK#7
try {
  var object = {
    valueOf: function() {
      return "%5E"
    },
    toString: function() {
      throw "error"
    }
  };
  decodeURI(object);
  throw new Test262Error('#7.1: var object = {valueOf: function() {return "%5E"}, toString: function() {throw "error"}}; decodeURI(object) throw "error". Actual: ' + (decodeURI(object)));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {return "%5E"}, toString: function() {throw "error"}}; decodeURI(object) throw "error". Actual: ' + (e));
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
  decodeURI(object);
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; decodeURI(object) throw TypeError. Actual: ' + (decodeURI(object)));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; decodeURI(object) throw TypeError. Actual: ' + (e));
  }
}
