// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and string values
esid: sec-abstract-relational-comparison
features: [BigInt]
---*/
assert.sameValue(0n <= '0', true, 'The result of (0n <= "0") is true');
assert.sameValue('0' <= 0n, true, 'The result of ("0" <= 0n) is true');
assert.sameValue(0n <= '1', true, 'The result of (0n <= "1") is true');
assert.sameValue('0' <= 1n, true, 'The result of ("0" <= 1n) is true');
assert.sameValue(1n <= '0', false, 'The result of (1n <= "0") is false');
assert.sameValue('1' <= 0n, false, 'The result of ("1" <= 0n) is false');
assert.sameValue(0n <= '', true, 'The result of (0n <= "") is true');
assert.sameValue('' <= 0n, true, 'The result of ("" <= 0n) is true');
assert.sameValue(0n <= '1', true, 'The result of (0n <= "1") is true');
assert.sameValue('' <= 1n, true, 'The result of ("" <= 1n) is true');
assert.sameValue(1n <= '', false, 'The result of (1n <= "") is false');
assert.sameValue('1' <= 0n, false, 'The result of ("1" <= 0n) is false');
assert.sameValue(1n <= '1', true, 'The result of (1n <= "1") is true');
assert.sameValue('1' <= 1n, true, 'The result of ("1" <= 1n) is true');
assert.sameValue(1n <= '-1', false, 'The result of (1n <= "-1") is false');
assert.sameValue('1' <= -1n, false, 'The result of ("1" <= -1n) is false');
assert.sameValue(-1n <= '1', true, 'The result of (-1n <= "1") is true');
assert.sameValue('-1' <= 1n, true, 'The result of ("-1" <= 1n) is true');
assert.sameValue(-1n <= '-1', true, 'The result of (-1n <= "-1") is true');
assert.sameValue('-1' <= -1n, true, 'The result of ("-1" <= -1n) is true');

assert.sameValue(
  9007199254740993n <= '9007199254740992',
  false,
  'The result of (9007199254740993n <= "9007199254740992") is false'
);

assert.sameValue(
  '9007199254740993' <= 9007199254740992n,
  false,
  'The result of ("9007199254740993" <= 9007199254740992n) is false'
);

assert.sameValue(
  -9007199254740992n <= '-9007199254740993',
  false,
  'The result of (-9007199254740992n <= "-9007199254740993") is false'
);

assert.sameValue(
  '-9007199254740992' <= -9007199254740993n,
  false,
  'The result of ("-9007199254740992" <= -9007199254740993n) is false'
);
