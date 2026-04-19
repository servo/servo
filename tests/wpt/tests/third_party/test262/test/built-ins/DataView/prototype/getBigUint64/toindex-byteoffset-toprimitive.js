// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ToIndex conversions on byteOffset
esid: sec-dataview.prototype.getbiguint64
features: [ArrayBuffer, BigInt, DataView, DataView.prototype.setUint8, Symbol.toPrimitive, computed-property-names]
---*/

var buffer = new ArrayBuffer(12);
var sample = new DataView(buffer, 0);
sample.setUint8(0, 0x27);
sample.setUint8(1, 0x02);
sample.setUint8(2, 0x06);
sample.setUint8(3, 0x02);
sample.setUint8(4, 0x80);
sample.setUint8(5, 0x00);
sample.setUint8(6, 0x80);
sample.setUint8(7, 0x01);
sample.setUint8(8, 0x7f);
sample.setUint8(9, 0x00);
sample.setUint8(10, 0x01);
sample.setUint8(11, 0x02);

function err() {
  throw new Test262Error();
}

function MyError() {}

assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: function() {
    return 1;
  },
  valueOf: err,
  toString: err
}), 0x20602800080017fn, "ToPrimitive: @@toPrimitive takes precedence");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return 1;
  },
  toString: err
}), 0x20602800080017fn, "ToPrimitive: valueOf takes precedence over toString");
assert.sameValue(sample.getBigUint64({
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: toString with no valueOf");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: undefined,
  valueOf: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip @@toPrimitive when it's undefined");
assert.sameValue(sample.getBigUint64({
  [Symbol.toPrimitive]: null,
  valueOf: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip @@toPrimitive when it's null");
assert.sameValue(sample.getBigUint64({
  valueOf: null,
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigUint64({
  valueOf: 1,
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigUint64({
  valueOf: {},
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return {};
  },
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it returns an object");
assert.sameValue(sample.getBigUint64({
  valueOf: function() {
    return Object(12345);
  },
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it returns an object");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    [Symbol.toPrimitive]: 1
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    [Symbol.toPrimitive]: {}
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    [Symbol.toPrimitive]: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(MyError, function() {
  sample.getBigUint64({
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from @@toPrimitive");
assert.throws(MyError, function() {
  sample.getBigUint64({
    valueOf: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from valueOf");
assert.throws(MyError, function() {
  sample.getBigUint64({
    toString: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from toString");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    valueOf: null,
    toString: null
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    valueOf: 1,
    toString: 1
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    valueOf: {},
    toString: {}
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    valueOf: function() {
      return Object(1);
    },
    toString: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigUint64({
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
