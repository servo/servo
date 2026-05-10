// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Relational comparison of BigInt and string values
esid: sec-abstract-relational-comparison
features: [BigInt]
---*/
assert.sameValue(0n < '0', false, 'The result of (0n < "0") is false');
assert.sameValue('0' < 0n, false, 'The result of ("0" < 0n) is false');
assert.sameValue(0n < '1', true, 'The result of (0n < "1") is true');
assert.sameValue('0' < 1n, true, 'The result of ("0" < 1n) is true');
assert.sameValue(1n < '0', false, 'The result of (1n < "0") is false');
assert.sameValue('1' < 0n, false, 'The result of ("1" < 0n) is false');
assert.sameValue(0n < '', false, 'The result of (0n < "") is false');
assert.sameValue('' < 0n, false, 'The result of ("" < 0n) is false');
assert.sameValue(0n < '1', true, 'The result of (0n < "1") is true');
assert.sameValue('' < 1n, true, 'The result of ("" < 1n) is true');
assert.sameValue(1n < '', false, 'The result of (1n < "") is false');
assert.sameValue('1' < 0n, false, 'The result of ("1" < 0n) is false');
assert.sameValue(1n < '1', false, 'The result of (1n < "1") is false');
assert.sameValue('1' < 1n, false, 'The result of ("1" < 1n) is false');
assert.sameValue(1n < '-1', false, 'The result of (1n < "-1") is false');
assert.sameValue('1' < -1n, false, 'The result of ("1" < -1n) is false');
assert.sameValue(-1n < '1', true, 'The result of (-1n < "1") is true');
assert.sameValue('-1' < 1n, true, 'The result of ("-1" < 1n) is true');
assert.sameValue(-1n < '-1', false, 'The result of (-1n < "-1") is false');
assert.sameValue('-1' < -1n, false, 'The result of ("-1" < -1n) is false');
assert.sameValue('0x10' < 15n, false, 'The result of ("0x10" < 15n) is false');
assert.sameValue('0x10' < 16n, false, 'The result of ("0x10" < 16n) is false');
assert.sameValue('0x10' < 17n, true, 'The result of ("0x10" < 17n) is true');
assert.sameValue('0o10' < 7n, false, 'The result of ("0o10" < 7n) is false');
assert.sameValue('0o10' < 8n, false, 'The result of ("0o10" < 8n) is false');
assert.sameValue('0o10' < 9n, true, 'The result of ("0o10" < 9n) is true');
assert.sameValue('0b10' < 1n, false, 'The result of ("0b10" < 1n) is false');
assert.sameValue('0b10' < 2n, false, 'The result of ("0b10" < 2n) is false');
assert.sameValue('0b10' < 3n, true, 'The result of ("0b10" < 3n) is true');

assert.sameValue(
  9007199254740993n < '9007199254740992',
  false,
  'The result of (9007199254740993n < "9007199254740992") is false'
);

assert.sameValue(
  '9007199254740993' < 9007199254740992n,
  false,
  'The result of ("9007199254740993" < 9007199254740992n) is false'
);

assert.sameValue(
  -9007199254740992n < '-9007199254740993',
  false,
  'The result of (-9007199254740992n < "-9007199254740993") is false'
);

assert.sameValue(
  '-9007199254740992' < -9007199254740993n,
  false,
  'The result of ("-9007199254740992" < -9007199254740993n) is false'
);
