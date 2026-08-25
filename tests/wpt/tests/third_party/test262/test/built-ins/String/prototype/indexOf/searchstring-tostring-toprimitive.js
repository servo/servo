// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for searchString parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  3. Let searchStr be ? ToString(searchString).
features: [Symbol.toPrimitive, computed-property-names]
---*/

function err() {
  throw new Test262Error();
}

function MyError() {}

assert.sameValue("__foo__".indexOf({
  [Symbol.toPrimitive]: function() {
    return "foo";
  },
  toString: err,
  valueOf: err
}), 2, "ToPrimitive: @@toPrimitive takes precedence");
assert.sameValue("__foo__".indexOf({
  toString: function() {
    return "foo";
  },
  valueOf: err
}), 2, "ToPrimitive: toString takes precedence over valueOf");
assert.sameValue("__foo__".indexOf({
  [Symbol.toPrimitive]: undefined,
  toString: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip @@toPrimitive when it's undefined");
assert.sameValue("__foo__".indexOf({
  [Symbol.toPrimitive]: null,
  toString: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip @@toPrimitive when it's null");
assert.sameValue("__foo__".indexOf({
  toString: null,
  valueOf: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip toString when it's not callable");
assert.sameValue("__foo__".indexOf({
  toString: 1,
  valueOf: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip toString when it's not callable");
assert.sameValue("__foo__".indexOf({
  toString: {},
  valueOf: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip toString when it's not callable");
assert.sameValue("__foo__".indexOf({
  toString: function() {
    return {};
  },
  valueOf: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip toString when it returns an object");
assert.sameValue("__foo__".indexOf({
  toString: function() {
    return Object(12345);
  },
  valueOf: function() {
    return "foo";
  }
}), 2, "ToPrimitive: skip toString when it returns an object");
assert.throws(TypeError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: 1
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: {}
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(TypeError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(MyError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from @@toPrimitive");
assert.throws(MyError, function() {
  "".indexOf({
    valueOf: function() {
      throw new MyError();
    },
    toString: null
  });
}, "ToPrimitive: propagate errors from valueOf");
assert.throws(MyError, function() {
  "".indexOf({
    toString: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from toString");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: null,
    toString: null
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: 1,
    toString: 1
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: {},
    toString: {}
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: function() {
      return Object(1);
    },
    toString: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
