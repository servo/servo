// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-encodeuri-uri
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return "^"
  }
};
if (encodeURI(object) !== "%5Bobject%20Object%5D") {
  throw new Test262Error('#1: var object = {valueOf: function() {return "^"}}; encodeURI(object) === %5Bobject%20Object%5D. Actual: ' + (encodeURI(object)));
}

//CHECK#2
var object = {
  valueOf: function() {
    return ""
  },
  toString: function() {
    return "^"
  }
};
if (encodeURI(object) !== "%5E") {
  throw new Test262Error('#2: var object = {valueOf: function() {return ""}, toString: function() {return "^"}}; encodeURI(object) === "%5E". Actual: ' + (encodeURI(object)));
}

//CHECK#3
var object = {
  valueOf: function() {
    return "^"
  },
  toString: function() {
    return {}
  }
};
if (encodeURI(object) !== "%5E") {
  throw new Test262Error('#3: var object = {valueOf: function() {return "^"}, toString: function() {return {}}}; encodeURI(object) === "%5E". Actual: ' + (encodeURI(object)));
}

//CHECK#4
try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return "^"
    }
  };
  if (encodeURI(object) !== "%5E") {
    throw new Test262Error('#4.1: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURI(object) === "%5E". Actual: ' + (encodeURI(object)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURI(object) not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURI(object) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return "^"
  }
};
if (encodeURI(object) !== "%5E") {
  throw new Test262Error('#5: var object = {toString: function() {return "^"}}; encodeURI(object) === "%5E". Actual: ' + (encodeURI(object)));
}

//CHECK#6
var object = {
  valueOf: function() {
    return {}
  },
  toString: function() {
    return "^"
  }
}
if (encodeURI(object) !== "%5E") {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return "^"}}; encodeURI(object) === "%5E". Actual: ' + (encodeURI(object)));
}

//CHECK#7
try {
  var object = {
    valueOf: function() {
      return "^"
    },
    toString: function() {
      throw "error"
    }
  };
  encodeURI(object);
  throw new Test262Error('#7.1: var object = {valueOf: function() {return "^"}, toString: function() {throw "error"}}; encodeURI(object) throw "error". Actual: ' + (encodeURI(object)));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {return "^"}, toString: function() {throw "error"}}; encodeURI(object) throw "error". Actual: ' + (e));
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
  encodeURI(object);
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; encodeURI(object) throw TypeError. Actual: ' + (encodeURI(object)));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; encodeURI(object) throw TypeError. Actual: ' + (e));
  }
}
