// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: BigInt.asUintN type coercion for bits parameter
esid: sec-bigint.asuintn
info: |
  BigInt.asUintN ( bits, bigint )

  1. Let bits be ? ToIndex(bits).
features: [BigInt]
---*/

assert.sameValue(BigInt.asUintN(0, 1n), 0n);
assert.sameValue(BigInt.asUintN(1, 1n), 1n);
assert.sameValue(BigInt.asUintN(-0.9, 1n), 0n, "ToIndex: truncate towards 0");
assert.sameValue(BigInt.asUintN(0.9, 1n), 0n, "ToIndex: truncate towards 0");
assert.sameValue(BigInt.asUintN(NaN, 1n), 0n, "ToIndex: NaN => 0");
assert.sameValue(BigInt.asUintN(undefined, 1n), 0n, "ToIndex: undefined => NaN => 0");
assert.sameValue(BigInt.asUintN(null, 1n), 0n, "ToIndex: null => 0");
assert.sameValue(BigInt.asUintN(false, 1n), 0n, "ToIndex: false => 0");
assert.sameValue(BigInt.asUintN(true, 1n), 1n, "ToIndex: true => 1");
assert.sameValue(BigInt.asUintN("0", 1n), 0n, "ToIndex: parse Number");
assert.sameValue(BigInt.asUintN("1", 1n), 1n, "ToIndex: parse Number");
assert.sameValue(BigInt.asUintN("", 1n), 0n, "ToIndex: parse Number => NaN => 0");
assert.sameValue(BigInt.asUintN("foo", 1n), 0n, "ToIndex: parse Number => NaN => 0");
assert.sameValue(BigInt.asUintN("true", 1n), 0n, "ToIndex: parse Number => NaN => 0");
assert.sameValue(BigInt.asUintN(3, 10n), 2n);
assert.sameValue(BigInt.asUintN("3", 10n), 2n, "toIndex: parse Number");
assert.sameValue(BigInt.asUintN(3.9, 10n), 2n, "toIndex: truncate towards 0");
assert.sameValue(BigInt.asUintN("3.9", 10n), 2n, "toIndex: parse Number => truncate towards 0");
assert.sameValue(BigInt.asUintN([0], 1n), 0n, 'ToIndex: [0].toString() => "0" => 0');
assert.sameValue(BigInt.asUintN(["1"], 1n), 1n, 'ToIndex: ["1"].toString() => "1" => 1');
assert.sameValue(BigInt.asUintN({}, 1n), 0n,
  'ToIndex: ({}).toString() => "[object Object]" => NaN => 0');
assert.sameValue(BigInt.asUintN([], 1n), 0n, 'ToIndex: [].toString() => "" => NaN => 0');
