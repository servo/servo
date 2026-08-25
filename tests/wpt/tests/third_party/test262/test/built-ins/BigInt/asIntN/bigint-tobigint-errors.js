// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asIntN type coercion for bigint parameter
esid: sec-bigint.asintn
info: |
  BigInt.asIntN ( bits, bigint )

  2. Let bigint ? ToBigInt(bigint).
features: [BigInt, computed-property-names, Symbol, Symbol.toPrimitive]
---*/
assert.sameValue(typeof BigInt, 'function');
assert.sameValue(typeof BigInt.asIntN, 'function');

assert.throws(TypeError, function () {
  BigInt.asIntN();
}, "ToBigInt: no argument => undefined => TypeError");
assert.throws(TypeError, function () {
  BigInt.asIntN(0);
}, "ToBigInt: no argument => undefined => TypeError");

assert.throws(TypeError, function() {
  BigInt.asIntN(0, undefined);
}, "ToBigInt: undefined => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    [Symbol.toPrimitive]: function() {
      return undefined;
    }
  });
}, "ToBigInt: @@toPrimitive => undefined => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    valueOf: function() {
      return undefined;
    }
  });
}, "ToBigInt: valueOf => undefined => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    toString: function() {
      return undefined;
    }
  });
}, "ToBigInt: toString => undefined => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, null);
}, "ToBigInt: null => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    [Symbol.toPrimitive]: function() {
      return null;
    }
  });
}, "ToBigInt: @@toPrimitive => null => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    valueOf: function() {
      return null;
    }
  });
}, "ToBigInt: valueOf => null => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    toString: function() {
      return null;
    }
  });
}, "ToBigInt: toString => null => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, 0);
}, "ToBigInt: Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, Object(0));
}, "ToBigInt: unbox object with internal slot => Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    [Symbol.toPrimitive]: function() {
      return 0;
    }
  });
}, "ToBigInt: @@toPrimitive => Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    valueOf: function() {
      return 0;
    }
  });
}, "ToBigInt: valueOf => Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    toString: function() {
      return 0;
    }
  });
}, "ToBigInt: toString => Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, NaN);
}, "ToBigInt: Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, Infinity);
}, "ToBigInt: Number => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, Symbol("1"));
}, "ToBigInt: Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, Object(Symbol("1")));
}, "ToBigInt: unbox object with internal slot => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    [Symbol.toPrimitive]: function() {
      return Symbol("1");
    }
  });
}, "ToBigInt: @@toPrimitive => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    valueOf: function() {
      return Symbol("1");
    }
  });
}, "ToBigInt: valueOf => Symbol => TypeError");
assert.throws(TypeError, function() {
  BigInt.asIntN(0, {
    toString: function() {
      return Symbol("1");
    }
  });
}, "ToBigInt: toString => Symbol => TypeError");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "a");
}, "ToBigInt: unparseable BigInt");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "0b2");
}, "ToBigInt: unparseable BigInt binary");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, Object("0b2"));
}, "ToBigInt: unbox object with internal slot => unparseable BigInt binary");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, {
    [Symbol.toPrimitive]: function() {
      return "0b2";
    }
  });
}, "ToBigInt: @@toPrimitive => unparseable BigInt binary");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, {
    valueOf: function() {
      return "0b2";
    }
  });
}, "ToBigInt: valueOf => unparseable BigInt binary");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, {
    toString: function() {
      return "0b2";
    }
  });
}, "ToBigInt: toString => unparseable BigInt binary");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "   0b2   ");
}, "ToBigInt: unparseable BigInt with leading/trailing whitespace");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "0o8");
}, "ToBigInt: unparseable BigInt octal");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "0xg");
}, "ToBigInt: unparseable BigInt hex");
assert.throws(SyntaxError, function() {
  BigInt.asIntN(0, "1n");
}, "ToBigInt: unparseable BigInt due to literal suffix");
