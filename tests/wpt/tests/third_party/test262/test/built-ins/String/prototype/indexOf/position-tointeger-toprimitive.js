// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for position parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  4. Let pos be ? ToInteger(position).
features: [Symbol.toPrimitive, computed-property-names]
---*/

function err() {
  throw new Test262Error();
}

function MyError() {}

assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return 1;
  },
  valueOf: err,
  toString: err
}), 1, "ToPrimitive: @@toPrimitive takes precedence");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return 1;
  },
  toString: err
}), 1, "ToPrimitive: valueOf takes precedence over toString");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: toString with no valueOf");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: undefined,
  valueOf: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip @@toPrimitive when it's undefined");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: null,
  valueOf: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip @@toPrimitive when it's null");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: null,
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: 1,
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: {},
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return {};
  },
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip valueOf when it returns an object");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return Object(12345);
  },
  toString: function() {
    return 1;
  }
}), 1, "ToPrimitive: skip valueOf when it returns an object");
assert.throws(TypeError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: 1
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: {}
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(TypeError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(MyError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from @@toPrimitive");
assert.throws(MyError, function() {
  "".indexOf("", {
    valueOf: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from valueOf");
assert.throws(MyError, function() {
  "".indexOf("", {
    toString: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from toString");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: null,
    toString: null
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: 1,
    toString: 1
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: {},
    toString: {}
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: function() {
      return Object(1);
    },
    toString: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
