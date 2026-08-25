// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict equality comparison of BigInt values and non-primitive objects
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(0n === Object(0n), false, 'The result of (0n === Object(0n)) is false');
assert.sameValue(Object(0n) === 0n, false, 'The result of (Object(0n) === 0n) is false');
assert.sameValue(0n === Object(1n), false, 'The result of (0n === Object(1n)) is false');
assert.sameValue(Object(1n) === 0n, false, 'The result of (Object(1n) === 0n) is false');
assert.sameValue(1n === Object(0n), false, 'The result of (1n === Object(0n)) is false');
assert.sameValue(Object(0n) === 1n, false, 'The result of (Object(0n) === 1n) is false');
assert.sameValue(1n === Object(1n), false, 'The result of (1n === Object(1n)) is false');
assert.sameValue(Object(1n) === 1n, false, 'The result of (Object(1n) === 1n) is false');
assert.sameValue(2n === Object(0n), false, 'The result of (2n === Object(0n)) is false');
assert.sameValue(Object(0n) === 2n, false, 'The result of (Object(0n) === 2n) is false');
assert.sameValue(2n === Object(1n), false, 'The result of (2n === Object(1n)) is false');
assert.sameValue(Object(1n) === 2n, false, 'The result of (Object(1n) === 2n) is false');
assert.sameValue(2n === Object(2n), false, 'The result of (2n === Object(2n)) is false');
assert.sameValue(Object(2n) === 2n, false, 'The result of (Object(2n) === 2n) is false');
assert.sameValue(0n === {}, false, 'The result of (0n === {}) is false');
assert.sameValue({} === 0n, false, 'The result of (({}) === 0n) is false');

assert.sameValue(0n === {
  valueOf: function() {
    return 0n;
  }
}, false, 'The result of (0n === {valueOf: function() {return 0n;}}) is false');

assert.sameValue({
  valueOf: function() {
    return 0n;
  }
} === 0n, false, 'The result of (({valueOf: function() {return 0n;}}) === 0n) is false');

assert.sameValue(0n === {
  valueOf: function() {
    return 1n;
  }
}, false, 'The result of (0n === {valueOf: function() {return 1n;}}) is false');

assert.sameValue({
  valueOf: function() {
    return 1n;
  }
} === 0n, false, 'The result of (({valueOf: function() {return 1n;}}) === 0n) is false');

assert.sameValue(0n === {
  toString: function() {
    return '0';
  }
}, false, 'The result of (0n === {toString: function() {return "0";}}) is false');

assert.sameValue({
  toString: function() {
    return '0';
  }
} === 0n, false, 'The result of (({toString: function() {return "0";}}) === 0n) is false');

assert.sameValue(0n === {
  toString: function() {
    return '1';
  }
}, false, 'The result of (0n === {toString: function() {return "1";}}) is false');

assert.sameValue({
  toString: function() {
    return '1';
  }
} === 0n, false, 'The result of (({toString: function() {return "1";}}) === 0n) is false');

assert.sameValue(900719925474099101n === {
  valueOf: function() {
    return 900719925474099101n;
  }
}, false, 'The result of (900719925474099101n === {valueOf: function() {return 900719925474099101n;}}) is false');

assert.sameValue({
  valueOf: function() {
    return 900719925474099101n;
  }
} === 900719925474099101n, false, 'The result of (({valueOf: function() {return 900719925474099101n;}}) === 900719925474099101n) is false');

assert.sameValue(900719925474099101n === {
  valueOf: function() {
    return 900719925474099102n;
  }
}, false, 'The result of (900719925474099101n === {valueOf: function() {return 900719925474099102n;}}) is false');

assert.sameValue({
  valueOf: function() {
    return 900719925474099102n;
  }
} === 900719925474099101n, false, 'The result of (({valueOf: function() {return 900719925474099102n;}}) === 900719925474099101n) is false');

assert.sameValue(900719925474099101n === {
  toString: function() {
    return '900719925474099101';
  }
}, false, 'The result of (900719925474099101n === {toString: function() {return "900719925474099101";}}) is false');

assert.sameValue({
  toString: function() {
    return '900719925474099101';
  }
} === 900719925474099101n, false, 'The result of (({toString: function() {return "900719925474099101";}}) === 900719925474099101n) is false');

assert.sameValue(900719925474099101n === {
  toString: function() {
    return '900719925474099102';
  }
}, false, 'The result of (900719925474099101n === {toString: function() {return "900719925474099102";}}) is false');

assert.sameValue({
  toString: function() {
    return '900719925474099102';
  }
} === 900719925474099101n, false, 'The result of (({toString: function() {return "900719925474099102";}}) === 900719925474099101n) is false');
