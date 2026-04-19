// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Strict inequality comparison of BigInt and String values
esid: sec-strict-equality-comparison
info: |
  1. If Type(x) is different from Type(y), return false.

features: [BigInt]
---*/
assert.sameValue(0n !== '', true, 'The result of (0n !== "") is true');
assert.sameValue('' !== 0n, true, 'The result of ("" !== 0n) is true');
assert.sameValue(0n !== '-0', true, 'The result of (0n !== "-0") is true');
assert.sameValue('-0' !== 0n, true, 'The result of ("-0" !== 0n) is true');
assert.sameValue(0n !== '0', true, 'The result of (0n !== "0") is true');
assert.sameValue('0' !== 0n, true, 'The result of ("0" !== 0n) is true');
assert.sameValue(0n !== '-1', true, 'The result of (0n !== "-1") is true');
assert.sameValue('-1' !== 0n, true, 'The result of ("-1" !== 0n) is true');
assert.sameValue(0n !== '1', true, 'The result of (0n !== "1") is true');
assert.sameValue('1' !== 0n, true, 'The result of ("1" !== 0n) is true');
assert.sameValue(0n !== 'foo', true, 'The result of (0n !== "foo") is true');
assert.sameValue('foo' !== 0n, true, 'The result of ("foo" !== 0n) is true');
assert.sameValue(1n !== '', true, 'The result of (1n !== "") is true');
assert.sameValue('' !== 1n, true, 'The result of ("" !== 1n) is true');
assert.sameValue(1n !== '-0', true, 'The result of (1n !== "-0") is true');
assert.sameValue('-0' !== 1n, true, 'The result of ("-0" !== 1n) is true');
assert.sameValue(1n !== '0', true, 'The result of (1n !== "0") is true');
assert.sameValue('0' !== 1n, true, 'The result of ("0" !== 1n) is true');
assert.sameValue(1n !== '-1', true, 'The result of (1n !== "-1") is true');
assert.sameValue('-1' !== 1n, true, 'The result of ("-1" !== 1n) is true');
assert.sameValue(1n !== '1', true, 'The result of (1n !== "1") is true');
assert.sameValue('1' !== 1n, true, 'The result of ("1" !== 1n) is true');
assert.sameValue(1n !== 'foo', true, 'The result of (1n !== "foo") is true');
assert.sameValue('foo' !== 1n, true, 'The result of ("foo" !== 1n) is true');
assert.sameValue(-1n !== '-', true, 'The result of (-1n !== "-") is true');
assert.sameValue('-' !== -1n, true, 'The result of ("-" !== -1n) is true');
assert.sameValue(-1n !== '-0', true, 'The result of (-1n !== "-0") is true');
assert.sameValue('-0' !== -1n, true, 'The result of ("-0" !== -1n) is true');
assert.sameValue(-1n !== '-1', true, 'The result of (-1n !== "-1") is true');
assert.sameValue('-1' !== -1n, true, 'The result of ("-1" !== -1n) is true');
assert.sameValue(-1n !== '-foo', true, 'The result of (-1n !== "-foo") is true');
assert.sameValue('-foo' !== -1n, true, 'The result of ("-foo" !== -1n) is true');

assert.sameValue(
  900719925474099101n !== '900719925474099101',
  true,
  'The result of (900719925474099101n !== "900719925474099101") is true'
);

assert.sameValue(
  '900719925474099101' !== 900719925474099101n,
  true,
  'The result of ("900719925474099101" !== 900719925474099101n) is true'
);

assert.sameValue(
  900719925474099102n !== '900719925474099101',
  true,
  'The result of (900719925474099102n !== "900719925474099101") is true'
);

assert.sameValue(
  '900719925474099101' !== 900719925474099102n,
  true,
  'The result of ("900719925474099101" !== 900719925474099102n) is true'
);
