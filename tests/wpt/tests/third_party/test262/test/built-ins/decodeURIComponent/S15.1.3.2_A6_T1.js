// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-decodeuricomponent-encodeduricomponent
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return "%5E"
  }
};
if (decodeURIComponent(object) !== "[object Object]") {
  throw new Test262Error('#1: var object = {valueOf: function() {return "%5E"}}; decodeURIComponent(object) === [object Object]. Actual: ' + (decodeURIComponent(object)));
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
if (decodeURIComponent(object) !== "^") {
  throw new Test262Error('#2: var object = {valueOf: function() {return ""}, toString: function() {return "%5E"}}; decodeURIComponent(object) === "^". Actual: ' + (decodeURIComponent(object)));
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
if (decodeURIComponent(object) !== "^") {
  throw new Test262Error('#3: var object = {valueOf: function() {return "%5E"}, toString: function() {return {}}}; decodeURIComponent(object) === "^". Actual: ' + (decodeURIComponent(object)));
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
  if (decodeURIComponent(object) !== "^") {
    throw new Test262Error('#4.1: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURIComponent(object) === "^". Actual: ' + (decodeURIComponent(object)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURIComponent(object) not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {throw "error"}, toString: function() {return "%5E"}}; decodeURIComponent(object) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return "%5E"
  }
};
if (decodeURIComponent(object) !== "^") {
  throw new Test262Error('#5: var object = {toString: function() {return "%5E"}}; decodeURIComponent(object) === "^". Actual: ' + (decodeURIComponent(object)));
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
if (decodeURIComponent(object) !== "^") {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return "%5E"}}; decodeURIComponent(object) === "^". Actual: ' + (decodeURIComponent(object)));
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
  decodeURIComponent(object);
  throw new Test262Error('#7.1: var object = {valueOf: function() {return "%5E"}, toString: function() {throw "error"}}; decodeURIComponent(object) throw "error". Actual: ' + (decodeURIComponent(object)));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {return "%5E"}, toString: function() {throw "error"}}; decodeURIComponent(object) throw "error". Actual: ' + (e));
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
  decodeURIComponent(object);
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; decodeURIComponent(object) throw TypeError. Actual: ' + (decodeURIComponent(object)));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; decodeURIComponent(object) throw TypeError. Actual: ' + (e));
  }
}
