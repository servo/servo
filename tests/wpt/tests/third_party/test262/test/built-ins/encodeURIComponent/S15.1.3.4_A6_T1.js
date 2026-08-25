// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString
esid: sec-encodeuricomponent-uricomponent
description: If Type(value) is Object, evaluate ToPrimitive(value, String)
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return "^"
  }
};
if (encodeURIComponent(object) !== "%5Bobject%20Object%5D") {
  throw new Test262Error('#1: var object = {valueOf: function() {return "^"}}; encodeURIComponent(object) === %5Bobject%20Object%5D. Actual: ' + (encodeURIComponent(object)));
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
if (encodeURIComponent(object) !== "%5E") {
  throw new Test262Error('#2: var object = {valueOf: function() {return ""}, toString: function() {return "^"}}; encodeURIComponent(object) === "%5E". Actual: ' + (encodeURIComponent(object)));
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
if (encodeURIComponent(object) !== "%5E") {
  throw new Test262Error('#3: var object = {valueOf: function() {return "^"}, toString: function() {return {}}}; encodeURIComponent(object) === "%5E". Actual: ' + (encodeURIComponent(object)));
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
  if (encodeURIComponent(object) !== "%5E") {
    throw new Test262Error('#4.1: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURIComponent(object) === "%5E". Actual: ' + (encodeURIComponent(object)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURIComponent(object) not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {throw "error"}, toString: function() {return "^"}}; encodeURIComponent(object) not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return "^"
  }
};
if (encodeURIComponent(object) !== "%5E") {
  throw new Test262Error('#5: var object = {toString: function() {return "^"}}; encodeURIComponent(object) === "%5E". Actual: ' + (encodeURIComponent(object)));
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
if (encodeURIComponent(object) !== "%5E") {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return "^"}}; encodeURIComponent(object) === "%5E". Actual: ' + (encodeURIComponent(object)));
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
  encodeURIComponent(object);
  throw new Test262Error('#7.1: var object = {valueOf: function() {return "^"}, toString: function() {throw "error"}}; encodeURIComponent(object) throw "error". Actual: ' + (encodeURIComponent(object)));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {return "^"}, toString: function() {throw "error"}}; encodeURIComponent(object) throw "error". Actual: ' + (e));
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
  encodeURIComponent(object);
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; encodeURIComponent(object) throw TypeError. Actual: ' + (encodeURIComponent(object)));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; encodeURIComponent(object) throw TypeError. Actual: ' + (e));
  }
}
