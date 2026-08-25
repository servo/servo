// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator uses ToNumber
es5id: 9.7_A3.1_T4
description: Type(x) is Object
---*/

//CHECK#1
var object = {
  valueOf: function() {
    return 1
  }
};
if (String.fromCharCode(object).charCodeAt(0) !== 1) {
  throw new Test262Error('#1: var object = {valueOf: function() {return 1}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
}

//CHECK#2
var object = {
  valueOf: function() {
    return 1
  },
  toString: function() {
    return 0
  }
};
if (String.fromCharCode(object).charCodeAt(0) !== 1) {
  throw new Test262Error('#2: var object = {valueOf: function() {return 1}, toString: function() {return 0}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
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
if (String.fromCharCode(object).charCodeAt(0) !== 1) {
  throw new Test262Error('#3: var object = {valueOf: function() {return 1}, toString: function() {return {}}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
}

//CHECK#4
try {
  var object = {
    valueOf: function() {
      return 1
    },
    toString: function() {
      throw "error"
    }
  };
  if (String.fromCharCode(object).charCodeAt(0) !== 1) {
    throw new Test262Error('#4.1: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
  }
}
catch (e) {
  if (e === "error") {
    throw new Test262Error('#4.2: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; object not throw "error"');
  } else {
    throw new Test262Error('#4.3: var object = {valueOf: function() {return 1}, toString: function() {throw "error"}}; object not throw Error. Actual: ' + (e));
  }
}

//CHECK#5
var object = {
  toString: function() {
    return 1
  }
};
if (String.fromCharCode(object).charCodeAt(0) !== 1) {
  throw new Test262Error('#5: var object = {toString: function() {return 1}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
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
if (String.fromCharCode(object).charCodeAt(0) !== 1) {
  throw new Test262Error('#6: var object = {valueOf: function() {return {}}, toString: function() {return 1}}; String.fromCharCode(object).charCodeAt(0) === 1. Actual: ' + (String.fromCharCode(object).charCodeAt(0)));
}

//CHECK#7
try {
  var object = {
    valueOf: function() {
      throw "error"
    },
    toString: function() {
      return 1
    }
  };
  object >>> 0;
  throw new Test262Error('#7.1: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; object throw "error". Actual: ' + (object >>> 0));
}
catch (e) {
  if (e !== "error") {
    throw new Test262Error('#7.2: var object = {valueOf: function() {throw "error"}, toString: function() {return 1}}; object throw "error". Actual: ' + (e));
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
  object >>> 0;
  throw new Test262Error('#8.1: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; object throw TypeError. Actual: ' + (object >>> 0));
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#8.2: var object = {valueOf: function() {return {}}, toString: function() {return {}}}; object throw TypeError. Actual: ' + (e));
  }
}
