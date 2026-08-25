// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator use ToString from separator
esid: sec-array.prototype.join
description: >
    If Type(separator) is Object, evaluate ToPrimitive(separator,
    String)
---*/

var x = new Array(0, 1, 2, 3);
var object = {
  valueOf() {
    return "+"
  }
};

assert.sameValue(
  x.join(object),
  "0[object Object]1[object Object]2[object Object]3",
  'x.join({valueOf() {return "+"}}) must return "0[object Object]1[object Object]2[object Object]3"'
);

var object = {
  valueOf() {
    return "+"
  },
  toString() {
    return "*"
  }
};

assert.sameValue(
  x.join(object),
  "0*1*2*3",
  'x.join("{valueOf() {return "+"}, toString() {return "*"}}) must return "0*1*2*3"'
);

var object = {
  valueOf() {
    return "+"
  },
  toString() {
    return {}
  }
};

assert.sameValue(
  x.join(object),
  "0+1+2+3",
  'x.join({valueOf() {return "+"}, toString() {return {}}}) must return "0+1+2+3"'
);

try {
  var object = {
    valueOf() {
      throw "error"
    },
    toString() {
      return "*"
    }
  };

  assert.sameValue(
    x.join(object),
    "0*1*2*3",
    'x.join("{valueOf() {throw "error"}, toString() {return "*"}}) must return "0*1*2*3"'
  );
}
catch (e) {
  assert.notSameValue(e, "error", 'The value of e is not "error"');
}

var object = {
  toString() {
    return "*"
  }
};
assert.sameValue(x.join(object), "0*1*2*3", 'x.join({toString() {return "*"}}) must return "0*1*2*3"');

var object = {
  valueOf() {
    return {}
  },
  toString() {
    return "*"
  }
}

assert.sameValue(
  x.join(object),
  "0*1*2*3",
  'x.join({valueOf() {return {}}, toString() {return "*"}}) must return "0*1*2*3"'
);

try {
  var object = {
    valueOf() {
      return "+"
    },
    toString() {
      throw "error"
    }
  };
  x.join(object);
  throw new Test262Error('#7.1: var object = {valueOf() {return "+"}, toString() {throw "error"}}; x.join(object) throw "error". Actual: ' + (x.join(object)));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}

try {
  var object = {
    valueOf() {
      return {}
    },
    toString() {
      return {}
    }
  };
  x.join(object);
  throw new Test262Error('#8.1: var object = {valueOf() {return {}}, toString() {return {}}}; x.join(object) throw TypeError. Actual: ' + (x.join(object)));
}
catch (e) {
  assert.sameValue(
    e instanceof TypeError,
    true,
    'The result of evaluating (e instanceof TypeError) is expected to be true'
  );
}

try {
  var object = {
    toString() {
      throw "error"
    }
  };
  [].join(object);
  throw new Test262Error('#9.1: var object = {toString() {throw "error"}}; [].join(object) throw "error". Actual: ' + ([].join(object)));
}
catch (e) {
  assert.sameValue(e, "error", 'The value of e is expected to be "error"');
}
