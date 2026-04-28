// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asIntN type coercion for bits parameter
esid: sec-bigint.asintn
info: |
  BigInt.asIntN ( bits, bigint )

  1. Let bits be ? ToIndex(bits).
features: [BigInt, computed-property-names, Symbol, Symbol.toPrimitive]
---*/
assert.sameValue(typeof BigInt, 'function');
assert.sameValue(typeof BigInt.asIntN, 'function');

function err() {
  throw new Test262Error();
}

function MyError() {}

assert.sameValue(BigInt.asIntN({
  [Symbol.toPrimitive]: function() {
    return 1;
  },
  valueOf: err,
  toString: err
}, 1n), -1n, "ToPrimitive: @@toPrimitive takes precedence");
assert.sameValue(BigInt.asIntN({
  valueOf: function() {
    return 1;
  },
  toString: err
}, 1n), -1n, "ToPrimitive: valueOf takes precedence over toString");
assert.sameValue(BigInt.asIntN({
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: toString with no valueOf");
assert.sameValue(BigInt.asIntN({
  [Symbol.toPrimitive]: undefined,
  valueOf: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip @@toPrimitive when it's undefined");
assert.sameValue(BigInt.asIntN({
  [Symbol.toPrimitive]: null,
  valueOf: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip @@toPrimitive when it's null");
assert.sameValue(BigInt.asIntN({
  valueOf: null,
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(BigInt.asIntN({
  valueOf: 1,
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(BigInt.asIntN({
  valueOf: {},
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip valueOf when it's not callable");
assert.sameValue(BigInt.asIntN({
  valueOf: function() {
    return {};
  },
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip valueOf when it returns an object");
assert.sameValue(BigInt.asIntN({
  valueOf: function() {
    return Object(12345);
  },
  toString: function() {
    return 1;
  }
}, 1n), -1n, "ToPrimitive: skip valueOf when it returns an object");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: 1
  }, 0n);
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: {}
  }, 0n);
}, "ToPrimitive: throw when @@toPrimitive is not callable");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: function() {
      return Object(1);
    }
  }, 0n);
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: function() {
      return {};
    }
  }, 0n);
}, "ToPrimitive: throw when @@toPrimitive returns an object");
assert.throws(MyError, function() {
  BigInt.asIntN({
    [Symbol.toPrimitive]: function() {
      throw new MyError();
    }
  }, 0n);
}, "ToPrimitive: propagate errors from @@toPrimitive");
assert.throws(MyError, function() {
  BigInt.asIntN({
    valueOf: function() {
      throw new MyError();
    }
  }, 0n);
}, "ToPrimitive: propagate errors from valueOf");
assert.throws(MyError, function() {
  BigInt.asIntN({
    toString: function() {
      throw new MyError();
    }
  }, 0n);
}, "ToPrimitive: propagate errors from toString");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: null,
    toString: null
  }, 0n);
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: 1,
    toString: 1
  }, 0n);
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: {},
    toString: {}
  }, 0n);
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: function() {
      return Object(1);
    },
    toString: function() {
      return Object(1);
    }
  }, 0n);
}, "ToPrimitive: throw when skipping both valueOf and toString");
assert.throws(TypeError, function() {
  BigInt.asIntN({
    valueOf: function() {
      return {};
    },
    toString: function() {
      return {};
    }
  }, 0n);
}, "ToPrimitive: throw when skipping both valueOf and toString");
