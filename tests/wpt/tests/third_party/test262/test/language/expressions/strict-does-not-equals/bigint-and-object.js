// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict inequality comparison of BigInt values and non-primitive objects
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(0n !== Object(0n), true, 'The result of (0n !== Object(0n)) is true');
assert.sameValue(Object(0n) !== 0n, true, 'The result of (Object(0n) !== 0n) is true');
assert.sameValue(0n !== Object(1n), true, 'The result of (0n !== Object(1n)) is true');
assert.sameValue(Object(1n) !== 0n, true, 'The result of (Object(1n) !== 0n) is true');
assert.sameValue(1n !== Object(0n), true, 'The result of (1n !== Object(0n)) is true');
assert.sameValue(Object(0n) !== 1n, true, 'The result of (Object(0n) !== 1n) is true');
assert.sameValue(1n !== Object(1n), true, 'The result of (1n !== Object(1n)) is true');
assert.sameValue(Object(1n) !== 1n, true, 'The result of (Object(1n) !== 1n) is true');
assert.sameValue(2n !== Object(0n), true, 'The result of (2n !== Object(0n)) is true');
assert.sameValue(Object(0n) !== 2n, true, 'The result of (Object(0n) !== 2n) is true');
assert.sameValue(2n !== Object(1n), true, 'The result of (2n !== Object(1n)) is true');
assert.sameValue(Object(1n) !== 2n, true, 'The result of (Object(1n) !== 2n) is true');
assert.sameValue(2n !== Object(2n), true, 'The result of (2n !== Object(2n)) is true');
assert.sameValue(Object(2n) !== 2n, true, 'The result of (Object(2n) !== 2n) is true');
assert.sameValue(0n !== {}, true, 'The result of (0n !== {}) is true');
assert.sameValue({} !== 0n, true, 'The result of (({}) !== 0n) is true');

assert.sameValue(0n !== {
  valueOf: function() {
    return 0n;
  }
}, true, 'The result of (0n !== {valueOf: function() {return 0n;}}) is true');

assert.sameValue({
  valueOf: function() {
    return 0n;
  }
} !== 0n, true, 'The result of (({valueOf: function() {return 0n;}}) !== 0n) is true');

assert.sameValue(0n !== {
  valueOf: function() {
    return 1n;
  }
}, true, 'The result of (0n !== {valueOf: function() {return 1n;}}) is true');

assert.sameValue({
  valueOf: function() {
    return 1n;
  }
} !== 0n, true, 'The result of (({valueOf: function() {return 1n;}}) !== 0n) is true');

assert.sameValue(0n !== {
  toString: function() {
    return '0';
  }
}, true, 'The result of (0n !== {toString: function() {return "0";}}) is true');

assert.sameValue({
  toString: function() {
    return '0';
  }
} !== 0n, true, 'The result of (({toString: function() {return "0";}}) !== 0n) is true');

assert.sameValue(0n !== {
  toString: function() {
    return '1';
  }
}, true, 'The result of (0n !== {toString: function() {return "1";}}) is true');

assert.sameValue({
  toString: function() {
    return '1';
  }
} !== 0n, true, 'The result of (({toString: function() {return "1";}}) !== 0n) is true');

assert.sameValue(900719925474099101n !== {
  valueOf: function() {
    return 900719925474099101n;
  }
}, true, 'The result of (900719925474099101n !== {valueOf: function() {return 900719925474099101n;}}) is true');

assert.sameValue({
  valueOf: function() {
    return 900719925474099101n;
  }
} !== 900719925474099101n, true, 'The result of (({valueOf: function() {return 900719925474099101n;}}) !== 900719925474099101n) is true');

assert.sameValue(900719925474099101n !== {
  valueOf: function() {
    return 900719925474099102n;
  }
}, true, 'The result of (900719925474099101n !== {valueOf: function() {return 900719925474099102n;}}) is true');

assert.sameValue({
  valueOf: function() {
    return 900719925474099102n;
  }
} !== 900719925474099101n, true, 'The result of (({valueOf: function() {return 900719925474099102n;}}) !== 900719925474099101n) is true');

assert.sameValue(900719925474099101n !== {
  toString: function() {
    return '900719925474099101';
  }
}, true, 'The result of (900719925474099101n !== {toString: function() {return "900719925474099101";}}) is true');

assert.sameValue({
  toString: function() {
    return '900719925474099101';
  }
} !== 900719925474099101n, true, 'The result of (({toString: function() {return "900719925474099101";}}) !== 900719925474099101n) is true');

assert.sameValue(900719925474099101n !== {
  toString: function() {
    return '900719925474099102';
  }
}, true, 'The result of (900719925474099101n !== {toString: function() {return "900719925474099102";}}) is true');

assert.sameValue({
  toString: function() {
    return '900719925474099102';
  }
} !== 900719925474099101n, true, 'The result of (({toString: function() {return "900719925474099102";}}) !== 900719925474099101n) is true');
