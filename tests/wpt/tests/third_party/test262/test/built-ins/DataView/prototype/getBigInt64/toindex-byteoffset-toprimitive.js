// Copyright (C) 2017 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: ToIndex conversions on byteOffset
esid: sec-dataview.prototype.getbigint64
info: |
  DataView.prototype.getBigInt64 ( byteOffset [ , littleEndian ] )

  1. Let v be the this value.
  2. If littleEndian is not present, let littleEndian be undefined.
  3. Return ? GetViewValue(v, byteOffset, littleEndian, "Int64").

  24.3.1.1 GetViewValue ( view, requestIndex, isLittleEndian, type )

  ...
  4. Let getIndex be ? ToIndex(requestIndex).
  ...
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

assert.sameValue(sample.getBigInt64({
  [Symbol.toPrimitive]: function() {
    return 1;
  },
  valueOf: err,
  toString: err
}), 0x20602800080017fn, "ToPrimitive: @@toPrimitive takes precedence");
assert.sameValue(sample.getBigInt64({
  valueOf: function() {
    return 1;
  },
  toString: err
}), 0x20602800080017fn, "ToPrimitive: valueOf takes precedence over toString");
assert.sameValue(sample.getBigInt64({
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: toString with no valueOf");
assert.sameValue(sample.getBigInt64({
  [Symbol.toPrimitive]: undefined,
  valueOf: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip @@toPrimitive when it's undefined");
assert.sameValue(sample.getBigInt64({
  [Symbol.toPrimitive]: null,
  valueOf: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip @@toPrimitive when it's null");
assert.sameValue(sample.getBigInt64({
  valueOf: null,
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigInt64({
  valueOf: 1,
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigInt64({
  valueOf: {},
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(sample.getBigInt64({
  valueOf: function() {
    return {};
  },
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it returns an object");
assert.sameValue(sample.getBigInt64({
  valueOf: function() {
    return Object(12345);
  },
  toString: function() {
    return 1;
  }
}), 0x20602800080017fn, "ToPrimitive: skip valueOf when it returns an object");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: 1
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: {}
  });
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(MyError, function() {
  sample.getBigInt64({
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from @@toPrimitive");
assert.throws(MyError, function() {
  sample.getBigInt64({
    valueOf: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from valueOf");
assert.throws(MyError, function() {
  sample.getBigInt64({
    toString: function() {
      throw new MyError();
    }
  });
}, "ToPrimitive: propagate errors from toString");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: null,
    toString: null
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: 1,
    toString: 1
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: {},
    toString: {}
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: function() {
      return Object(1);
    },
    toString: function() {
      return Object(1);
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  sample.getBigInt64({
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  });
}, "ToPrimitive: throw when skipping both valueOf and toString");
