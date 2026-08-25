// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-subtraction-operator-minus-runtime-semantics-evaluation
description: BigInt subtraction arithmetic
features: [BigInt]
---*/
assert.sameValue(
  0xFEDCBA9876543210n - 0xFEDCBA9876543210n,
  0x0n,
  'The result of (0xFEDCBA9876543210n - 0xFEDCBA9876543210n) is 0x0n'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0xFEDCBA987654320Fn,
  0x1n,
  'The result of (0xFEDCBA9876543210n - 0xFEDCBA987654320Fn) is 0x1n'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0xFEDCBA98n,
  0xFEDCBA9777777778n,
  'The result of (0xFEDCBA9876543210n - 0xFEDCBA98n) is 0xFEDCBA9777777778n'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0xFEDCBA97n,
  0xFEDCBA9777777779n,
  'The result of (0xFEDCBA9876543210n - 0xFEDCBA97n) is 0xFEDCBA9777777779n'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0x1234n,
  0xFEDCBA9876541FDCn,
  'The result of (0xFEDCBA9876543210n - 0x1234n) is 0xFEDCBA9876541FDCn'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0x3n,
  0xFEDCBA987654320Dn,
  'The result of (0xFEDCBA9876543210n - 0x3n) is 0xFEDCBA987654320Dn'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0x2n,
  0xFEDCBA987654320En,
  'The result of (0xFEDCBA9876543210n - 0x2n) is 0xFEDCBA987654320En'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0x1n,
  0xFEDCBA987654320Fn,
  'The result of (0xFEDCBA9876543210n - 0x1n) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  0xFEDCBA9876543210n - 0x0n,
  0xFEDCBA9876543210n,
  'The result of (0xFEDCBA9876543210n - 0x0n) is 0xFEDCBA9876543210n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0x1n,
  0xFEDCBA9876543211n,
  'The result of (0xFEDCBA9876543210n - -0x1n) is 0xFEDCBA9876543211n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0x2n,
  0xFEDCBA9876543212n,
  'The result of (0xFEDCBA9876543210n - -0x2n) is 0xFEDCBA9876543212n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0x3n,
  0xFEDCBA9876543213n,
  'The result of (0xFEDCBA9876543210n - -0x3n) is 0xFEDCBA9876543213n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0x1234n,
  0xFEDCBA9876544444n,
  'The result of (0xFEDCBA9876543210n - -0x1234n) is 0xFEDCBA9876544444n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0xFEDCBA97n,
  0xFEDCBA997530ECA7n,
  'The result of (0xFEDCBA9876543210n - -0xFEDCBA97n) is 0xFEDCBA997530ECA7n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0xFEDCBA98n,
  0xFEDCBA997530ECA8n,
  'The result of (0xFEDCBA9876543210n - -0xFEDCBA98n) is 0xFEDCBA997530ECA8n'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0xFEDCBA987654320Fn,
  0x1FDB97530ECA8641Fn,
  'The result of (0xFEDCBA9876543210n - -0xFEDCBA987654320Fn) is 0x1FDB97530ECA8641Fn'
);

assert.sameValue(
  0xFEDCBA9876543210n - -0xFEDCBA9876543210n,
  0x1FDB97530ECA86420n,
  'The result of (0xFEDCBA9876543210n - -0xFEDCBA9876543210n) is 0x1FDB97530ECA86420n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0xFEDCBA9876543210n,
  -0x1n,
  'The result of (0xFEDCBA987654320Fn - 0xFEDCBA9876543210n) is -0x1n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0xFEDCBA987654320Fn,
  0x0n,
  'The result of (0xFEDCBA987654320Fn - 0xFEDCBA987654320Fn) is 0x0n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0xFEDCBA98n,
  0xFEDCBA9777777777n,
  'The result of (0xFEDCBA987654320Fn - 0xFEDCBA98n) is 0xFEDCBA9777777777n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0xFEDCBA97n,
  0xFEDCBA9777777778n,
  'The result of (0xFEDCBA987654320Fn - 0xFEDCBA97n) is 0xFEDCBA9777777778n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0x1234n,
  0xFEDCBA9876541FDBn,
  'The result of (0xFEDCBA987654320Fn - 0x1234n) is 0xFEDCBA9876541FDBn'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0x3n,
  0xFEDCBA987654320Cn,
  'The result of (0xFEDCBA987654320Fn - 0x3n) is 0xFEDCBA987654320Cn'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0x2n,
  0xFEDCBA987654320Dn,
  'The result of (0xFEDCBA987654320Fn - 0x2n) is 0xFEDCBA987654320Dn'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0x1n,
  0xFEDCBA987654320En,
  'The result of (0xFEDCBA987654320Fn - 0x1n) is 0xFEDCBA987654320En'
);

assert.sameValue(
  0xFEDCBA987654320Fn - 0x0n,
  0xFEDCBA987654320Fn,
  'The result of (0xFEDCBA987654320Fn - 0x0n) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0x1n,
  0xFEDCBA9876543210n,
  'The result of (0xFEDCBA987654320Fn - -0x1n) is 0xFEDCBA9876543210n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0x2n,
  0xFEDCBA9876543211n,
  'The result of (0xFEDCBA987654320Fn - -0x2n) is 0xFEDCBA9876543211n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0x3n,
  0xFEDCBA9876543212n,
  'The result of (0xFEDCBA987654320Fn - -0x3n) is 0xFEDCBA9876543212n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0x1234n,
  0xFEDCBA9876544443n,
  'The result of (0xFEDCBA987654320Fn - -0x1234n) is 0xFEDCBA9876544443n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0xFEDCBA97n,
  0xFEDCBA997530ECA6n,
  'The result of (0xFEDCBA987654320Fn - -0xFEDCBA97n) is 0xFEDCBA997530ECA6n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0xFEDCBA98n,
  0xFEDCBA997530ECA7n,
  'The result of (0xFEDCBA987654320Fn - -0xFEDCBA98n) is 0xFEDCBA997530ECA7n'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0xFEDCBA987654320Fn,
  0x1FDB97530ECA8641En,
  'The result of (0xFEDCBA987654320Fn - -0xFEDCBA987654320Fn) is 0x1FDB97530ECA8641En'
);

assert.sameValue(
  0xFEDCBA987654320Fn - -0xFEDCBA9876543210n,
  0x1FDB97530ECA8641Fn,
  'The result of (0xFEDCBA987654320Fn - -0xFEDCBA9876543210n) is 0x1FDB97530ECA8641Fn'
);

assert.sameValue(
  0xFEDCBA98n - 0xFEDCBA9876543210n,
  -0xFEDCBA9777777778n,
  'The result of (0xFEDCBA98n - 0xFEDCBA9876543210n) is -0xFEDCBA9777777778n'
);

assert.sameValue(
  0xFEDCBA98n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9777777777n,
  'The result of (0xFEDCBA98n - 0xFEDCBA987654320Fn) is -0xFEDCBA9777777777n'
);

assert.sameValue(
  0xFEDCBA98n - 0xFEDCBA98n,
  0x0n,
  'The result of (0xFEDCBA98n - 0xFEDCBA98n) is 0x0n'
);

assert.sameValue(
  0xFEDCBA98n - 0xFEDCBA97n,
  0x1n,
  'The result of (0xFEDCBA98n - 0xFEDCBA97n) is 0x1n'
);

assert.sameValue(
  0xFEDCBA98n - 0x1234n,
  0xFEDCA864n,
  'The result of (0xFEDCBA98n - 0x1234n) is 0xFEDCA864n'
);

assert.sameValue(
  0xFEDCBA98n - 0x3n,
  0xFEDCBA95n,
  'The result of (0xFEDCBA98n - 0x3n) is 0xFEDCBA95n'
);

assert.sameValue(
  0xFEDCBA98n - 0x2n,
  0xFEDCBA96n,
  'The result of (0xFEDCBA98n - 0x2n) is 0xFEDCBA96n'
);

assert.sameValue(
  0xFEDCBA98n - 0x1n,
  0xFEDCBA97n,
  'The result of (0xFEDCBA98n - 0x1n) is 0xFEDCBA97n'
);

assert.sameValue(
  0xFEDCBA98n - 0x0n,
  0xFEDCBA98n,
  'The result of (0xFEDCBA98n - 0x0n) is 0xFEDCBA98n'
);

assert.sameValue(
  0xFEDCBA98n - -0x1n,
  0xFEDCBA99n,
  'The result of (0xFEDCBA98n - -0x1n) is 0xFEDCBA99n'
);

assert.sameValue(
  0xFEDCBA98n - -0x2n,
  0xFEDCBA9An,
  'The result of (0xFEDCBA98n - -0x2n) is 0xFEDCBA9An'
);

assert.sameValue(
  0xFEDCBA98n - -0x3n,
  0xFEDCBA9Bn,
  'The result of (0xFEDCBA98n - -0x3n) is 0xFEDCBA9Bn'
);

assert.sameValue(
  0xFEDCBA98n - -0x1234n,
  0xFEDCCCCCn,
  'The result of (0xFEDCBA98n - -0x1234n) is 0xFEDCCCCCn'
);

assert.sameValue(
  0xFEDCBA98n - -0xFEDCBA97n,
  0x1FDB9752Fn,
  'The result of (0xFEDCBA98n - -0xFEDCBA97n) is 0x1FDB9752Fn'
);

assert.sameValue(
  0xFEDCBA98n - -0xFEDCBA98n,
  0x1FDB97530n,
  'The result of (0xFEDCBA98n - -0xFEDCBA98n) is 0x1FDB97530n'
);

assert.sameValue(
  0xFEDCBA98n - -0xFEDCBA987654320Fn,
  0xFEDCBA997530ECA7n,
  'The result of (0xFEDCBA98n - -0xFEDCBA987654320Fn) is 0xFEDCBA997530ECA7n'
);

assert.sameValue(
  0xFEDCBA98n - -0xFEDCBA9876543210n,
  0xFEDCBA997530ECA8n,
  'The result of (0xFEDCBA98n - -0xFEDCBA9876543210n) is 0xFEDCBA997530ECA8n'
);

assert.sameValue(
  0xFEDCBA97n - 0xFEDCBA9876543210n,
  -0xFEDCBA9777777779n,
  'The result of (0xFEDCBA97n - 0xFEDCBA9876543210n) is -0xFEDCBA9777777779n'
);

assert.sameValue(
  0xFEDCBA97n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9777777778n,
  'The result of (0xFEDCBA97n - 0xFEDCBA987654320Fn) is -0xFEDCBA9777777778n'
);

assert.sameValue(
  0xFEDCBA97n - 0xFEDCBA98n,
  -0x1n,
  'The result of (0xFEDCBA97n - 0xFEDCBA98n) is -0x1n'
);

assert.sameValue(
  0xFEDCBA97n - 0xFEDCBA97n,
  0x0n,
  'The result of (0xFEDCBA97n - 0xFEDCBA97n) is 0x0n'
);

assert.sameValue(
  0xFEDCBA97n - 0x1234n,
  0xFEDCA863n,
  'The result of (0xFEDCBA97n - 0x1234n) is 0xFEDCA863n'
);

assert.sameValue(
  0xFEDCBA97n - 0x3n,
  0xFEDCBA94n,
  'The result of (0xFEDCBA97n - 0x3n) is 0xFEDCBA94n'
);

assert.sameValue(
  0xFEDCBA97n - 0x2n,
  0xFEDCBA95n,
  'The result of (0xFEDCBA97n - 0x2n) is 0xFEDCBA95n'
);

assert.sameValue(
  0xFEDCBA97n - 0x1n,
  0xFEDCBA96n,
  'The result of (0xFEDCBA97n - 0x1n) is 0xFEDCBA96n'
);

assert.sameValue(
  0xFEDCBA97n - 0x0n,
  0xFEDCBA97n,
  'The result of (0xFEDCBA97n - 0x0n) is 0xFEDCBA97n'
);

assert.sameValue(
  0xFEDCBA97n - -0x1n,
  0xFEDCBA98n,
  'The result of (0xFEDCBA97n - -0x1n) is 0xFEDCBA98n'
);

assert.sameValue(
  0xFEDCBA97n - -0x2n,
  0xFEDCBA99n,
  'The result of (0xFEDCBA97n - -0x2n) is 0xFEDCBA99n'
);

assert.sameValue(
  0xFEDCBA97n - -0x3n,
  0xFEDCBA9An,
  'The result of (0xFEDCBA97n - -0x3n) is 0xFEDCBA9An'
);

assert.sameValue(
  0xFEDCBA97n - -0x1234n,
  0xFEDCCCCBn,
  'The result of (0xFEDCBA97n - -0x1234n) is 0xFEDCCCCBn'
);

assert.sameValue(
  0xFEDCBA97n - -0xFEDCBA97n,
  0x1FDB9752En,
  'The result of (0xFEDCBA97n - -0xFEDCBA97n) is 0x1FDB9752En'
);

assert.sameValue(
  0xFEDCBA97n - -0xFEDCBA98n,
  0x1FDB9752Fn,
  'The result of (0xFEDCBA97n - -0xFEDCBA98n) is 0x1FDB9752Fn'
);

assert.sameValue(
  0xFEDCBA97n - -0xFEDCBA987654320Fn,
  0xFEDCBA997530ECA6n,
  'The result of (0xFEDCBA97n - -0xFEDCBA987654320Fn) is 0xFEDCBA997530ECA6n'
);

assert.sameValue(
  0xFEDCBA97n - -0xFEDCBA9876543210n,
  0xFEDCBA997530ECA7n,
  'The result of (0xFEDCBA97n - -0xFEDCBA9876543210n) is 0xFEDCBA997530ECA7n'
);

assert.sameValue(
  0x1234n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876541FDCn,
  'The result of (0x1234n - 0xFEDCBA9876543210n) is -0xFEDCBA9876541FDCn'
);

assert.sameValue(
  0x1234n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9876541FDBn,
  'The result of (0x1234n - 0xFEDCBA987654320Fn) is -0xFEDCBA9876541FDBn'
);

assert.sameValue(
  0x1234n - 0xFEDCBA98n,
  -0xFEDCA864n,
  'The result of (0x1234n - 0xFEDCBA98n) is -0xFEDCA864n'
);

assert.sameValue(
  0x1234n - 0xFEDCBA97n,
  -0xFEDCA863n,
  'The result of (0x1234n - 0xFEDCBA97n) is -0xFEDCA863n'
);

assert.sameValue(0x1234n - 0x1234n, 0x0n, 'The result of (0x1234n - 0x1234n) is 0x0n');
assert.sameValue(0x1234n - 0x3n, 0x1231n, 'The result of (0x1234n - 0x3n) is 0x1231n');
assert.sameValue(0x1234n - 0x2n, 0x1232n, 'The result of (0x1234n - 0x2n) is 0x1232n');
assert.sameValue(0x1234n - 0x1n, 0x1233n, 'The result of (0x1234n - 0x1n) is 0x1233n');
assert.sameValue(0x1234n - 0x0n, 0x1234n, 'The result of (0x1234n - 0x0n) is 0x1234n');
assert.sameValue(0x1234n - -0x1n, 0x1235n, 'The result of (0x1234n - -0x1n) is 0x1235n');
assert.sameValue(0x1234n - -0x2n, 0x1236n, 'The result of (0x1234n - -0x2n) is 0x1236n');
assert.sameValue(0x1234n - -0x3n, 0x1237n, 'The result of (0x1234n - -0x3n) is 0x1237n');
assert.sameValue(0x1234n - -0x1234n, 0x2468n, 'The result of (0x1234n - -0x1234n) is 0x2468n');

assert.sameValue(
  0x1234n - -0xFEDCBA97n,
  0xFEDCCCCBn,
  'The result of (0x1234n - -0xFEDCBA97n) is 0xFEDCCCCBn'
);

assert.sameValue(
  0x1234n - -0xFEDCBA98n,
  0xFEDCCCCCn,
  'The result of (0x1234n - -0xFEDCBA98n) is 0xFEDCCCCCn'
);

assert.sameValue(
  0x1234n - -0xFEDCBA987654320Fn,
  0xFEDCBA9876544443n,
  'The result of (0x1234n - -0xFEDCBA987654320Fn) is 0xFEDCBA9876544443n'
);

assert.sameValue(
  0x1234n - -0xFEDCBA9876543210n,
  0xFEDCBA9876544444n,
  'The result of (0x1234n - -0xFEDCBA9876543210n) is 0xFEDCBA9876544444n'
);

assert.sameValue(
  0x3n - 0xFEDCBA9876543210n,
  -0xFEDCBA987654320Dn,
  'The result of (0x3n - 0xFEDCBA9876543210n) is -0xFEDCBA987654320Dn'
);

assert.sameValue(
  0x3n - 0xFEDCBA987654320Fn,
  -0xFEDCBA987654320Cn,
  'The result of (0x3n - 0xFEDCBA987654320Fn) is -0xFEDCBA987654320Cn'
);

assert.sameValue(
  0x3n - 0xFEDCBA98n,
  -0xFEDCBA95n,
  'The result of (0x3n - 0xFEDCBA98n) is -0xFEDCBA95n'
);

assert.sameValue(
  0x3n - 0xFEDCBA97n,
  -0xFEDCBA94n,
  'The result of (0x3n - 0xFEDCBA97n) is -0xFEDCBA94n'
);

assert.sameValue(0x3n - 0x1234n, -0x1231n, 'The result of (0x3n - 0x1234n) is -0x1231n');
assert.sameValue(0x3n - 0x3n, 0x0n, 'The result of (0x3n - 0x3n) is 0x0n');
assert.sameValue(0x3n - 0x2n, 0x1n, 'The result of (0x3n - 0x2n) is 0x1n');
assert.sameValue(0x3n - 0x1n, 0x2n, 'The result of (0x3n - 0x1n) is 0x2n');
assert.sameValue(0x3n - 0x0n, 0x3n, 'The result of (0x3n - 0x0n) is 0x3n');
assert.sameValue(0x3n - -0x1n, 0x4n, 'The result of (0x3n - -0x1n) is 0x4n');
assert.sameValue(0x3n - -0x2n, 0x5n, 'The result of (0x3n - -0x2n) is 0x5n');
assert.sameValue(0x3n - -0x3n, 0x6n, 'The result of (0x3n - -0x3n) is 0x6n');
assert.sameValue(0x3n - -0x1234n, 0x1237n, 'The result of (0x3n - -0x1234n) is 0x1237n');

assert.sameValue(
  0x3n - -0xFEDCBA97n,
  0xFEDCBA9An,
  'The result of (0x3n - -0xFEDCBA97n) is 0xFEDCBA9An'
);

assert.sameValue(
  0x3n - -0xFEDCBA98n,
  0xFEDCBA9Bn,
  'The result of (0x3n - -0xFEDCBA98n) is 0xFEDCBA9Bn'
);

assert.sameValue(
  0x3n - -0xFEDCBA987654320Fn,
  0xFEDCBA9876543212n,
  'The result of (0x3n - -0xFEDCBA987654320Fn) is 0xFEDCBA9876543212n'
);

assert.sameValue(
  0x3n - -0xFEDCBA9876543210n,
  0xFEDCBA9876543213n,
  'The result of (0x3n - -0xFEDCBA9876543210n) is 0xFEDCBA9876543213n'
);

assert.sameValue(
  0x2n - 0xFEDCBA9876543210n,
  -0xFEDCBA987654320En,
  'The result of (0x2n - 0xFEDCBA9876543210n) is -0xFEDCBA987654320En'
);

assert.sameValue(
  0x2n - 0xFEDCBA987654320Fn,
  -0xFEDCBA987654320Dn,
  'The result of (0x2n - 0xFEDCBA987654320Fn) is -0xFEDCBA987654320Dn'
);

assert.sameValue(
  0x2n - 0xFEDCBA98n,
  -0xFEDCBA96n,
  'The result of (0x2n - 0xFEDCBA98n) is -0xFEDCBA96n'
);

assert.sameValue(
  0x2n - 0xFEDCBA97n,
  -0xFEDCBA95n,
  'The result of (0x2n - 0xFEDCBA97n) is -0xFEDCBA95n'
);

assert.sameValue(0x2n - 0x1234n, -0x1232n, 'The result of (0x2n - 0x1234n) is -0x1232n');
assert.sameValue(0x2n - 0x3n, -0x1n, 'The result of (0x2n - 0x3n) is -0x1n');
assert.sameValue(0x2n - 0x2n, 0x0n, 'The result of (0x2n - 0x2n) is 0x0n');
assert.sameValue(0x2n - 0x1n, 0x1n, 'The result of (0x2n - 0x1n) is 0x1n');
assert.sameValue(0x2n - 0x0n, 0x2n, 'The result of (0x2n - 0x0n) is 0x2n');
assert.sameValue(0x2n - -0x1n, 0x3n, 'The result of (0x2n - -0x1n) is 0x3n');
assert.sameValue(0x2n - -0x2n, 0x4n, 'The result of (0x2n - -0x2n) is 0x4n');
assert.sameValue(0x2n - -0x3n, 0x5n, 'The result of (0x2n - -0x3n) is 0x5n');
assert.sameValue(0x2n - -0x1234n, 0x1236n, 'The result of (0x2n - -0x1234n) is 0x1236n');

assert.sameValue(
  0x2n - -0xFEDCBA97n,
  0xFEDCBA99n,
  'The result of (0x2n - -0xFEDCBA97n) is 0xFEDCBA99n'
);

assert.sameValue(
  0x2n - -0xFEDCBA98n,
  0xFEDCBA9An,
  'The result of (0x2n - -0xFEDCBA98n) is 0xFEDCBA9An'
);

assert.sameValue(
  0x2n - -0xFEDCBA987654320Fn,
  0xFEDCBA9876543211n,
  'The result of (0x2n - -0xFEDCBA987654320Fn) is 0xFEDCBA9876543211n'
);

assert.sameValue(
  0x2n - -0xFEDCBA9876543210n,
  0xFEDCBA9876543212n,
  'The result of (0x2n - -0xFEDCBA9876543210n) is 0xFEDCBA9876543212n'
);

assert.sameValue(
  0x1n - 0xFEDCBA9876543210n,
  -0xFEDCBA987654320Fn,
  'The result of (0x1n - 0xFEDCBA9876543210n) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  0x1n - 0xFEDCBA987654320Fn,
  -0xFEDCBA987654320En,
  'The result of (0x1n - 0xFEDCBA987654320Fn) is -0xFEDCBA987654320En'
);

assert.sameValue(
  0x1n - 0xFEDCBA98n,
  -0xFEDCBA97n,
  'The result of (0x1n - 0xFEDCBA98n) is -0xFEDCBA97n'
);

assert.sameValue(
  0x1n - 0xFEDCBA97n,
  -0xFEDCBA96n,
  'The result of (0x1n - 0xFEDCBA97n) is -0xFEDCBA96n'
);

assert.sameValue(0x1n - 0x1234n, -0x1233n, 'The result of (0x1n - 0x1234n) is -0x1233n');
assert.sameValue(0x1n - 0x3n, -0x2n, 'The result of (0x1n - 0x3n) is -0x2n');
assert.sameValue(0x1n - 0x2n, -0x1n, 'The result of (0x1n - 0x2n) is -0x1n');
assert.sameValue(0x1n - 0x1n, 0x0n, 'The result of (0x1n - 0x1n) is 0x0n');
assert.sameValue(0x1n - 0x0n, 0x1n, 'The result of (0x1n - 0x0n) is 0x1n');
assert.sameValue(0x1n - -0x1n, 0x2n, 'The result of (0x1n - -0x1n) is 0x2n');
assert.sameValue(0x1n - -0x2n, 0x3n, 'The result of (0x1n - -0x2n) is 0x3n');
assert.sameValue(0x1n - -0x3n, 0x4n, 'The result of (0x1n - -0x3n) is 0x4n');
assert.sameValue(0x1n - -0x1234n, 0x1235n, 'The result of (0x1n - -0x1234n) is 0x1235n');

assert.sameValue(
  0x1n - -0xFEDCBA97n,
  0xFEDCBA98n,
  'The result of (0x1n - -0xFEDCBA97n) is 0xFEDCBA98n'
);

assert.sameValue(
  0x1n - -0xFEDCBA98n,
  0xFEDCBA99n,
  'The result of (0x1n - -0xFEDCBA98n) is 0xFEDCBA99n'
);

assert.sameValue(
  0x1n - -0xFEDCBA987654320Fn,
  0xFEDCBA9876543210n,
  'The result of (0x1n - -0xFEDCBA987654320Fn) is 0xFEDCBA9876543210n'
);

assert.sameValue(
  0x1n - -0xFEDCBA9876543210n,
  0xFEDCBA9876543211n,
  'The result of (0x1n - -0xFEDCBA9876543210n) is 0xFEDCBA9876543211n'
);

assert.sameValue(
  0x0n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876543210n,
  'The result of (0x0n - 0xFEDCBA9876543210n) is -0xFEDCBA9876543210n'
);

assert.sameValue(
  0x0n - 0xFEDCBA987654320Fn,
  -0xFEDCBA987654320Fn,
  'The result of (0x0n - 0xFEDCBA987654320Fn) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  0x0n - 0xFEDCBA98n,
  -0xFEDCBA98n,
  'The result of (0x0n - 0xFEDCBA98n) is -0xFEDCBA98n'
);

assert.sameValue(
  0x0n - 0xFEDCBA97n,
  -0xFEDCBA97n,
  'The result of (0x0n - 0xFEDCBA97n) is -0xFEDCBA97n'
);

assert.sameValue(0x0n - 0x1234n, -0x1234n, 'The result of (0x0n - 0x1234n) is -0x1234n');
assert.sameValue(0x0n - 0x3n, -0x3n, 'The result of (0x0n - 0x3n) is -0x3n');
assert.sameValue(0x0n - 0x2n, -0x2n, 'The result of (0x0n - 0x2n) is -0x2n');
assert.sameValue(0x0n - 0x1n, -0x1n, 'The result of (0x0n - 0x1n) is -0x1n');
assert.sameValue(0x0n - 0x0n, 0x0n, 'The result of (0x0n - 0x0n) is 0x0n');
assert.sameValue(0x0n - -0x1n, 0x1n, 'The result of (0x0n - -0x1n) is 0x1n');
assert.sameValue(0x0n - -0x2n, 0x2n, 'The result of (0x0n - -0x2n) is 0x2n');
assert.sameValue(0x0n - -0x3n, 0x3n, 'The result of (0x0n - -0x3n) is 0x3n');
assert.sameValue(0x0n - -0x1234n, 0x1234n, 'The result of (0x0n - -0x1234n) is 0x1234n');

assert.sameValue(
  0x0n - -0xFEDCBA97n,
  0xFEDCBA97n,
  'The result of (0x0n - -0xFEDCBA97n) is 0xFEDCBA97n'
);

assert.sameValue(
  0x0n - -0xFEDCBA98n,
  0xFEDCBA98n,
  'The result of (0x0n - -0xFEDCBA98n) is 0xFEDCBA98n'
);

assert.sameValue(
  0x0n - -0xFEDCBA987654320Fn,
  0xFEDCBA987654320Fn,
  'The result of (0x0n - -0xFEDCBA987654320Fn) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  0x0n - -0xFEDCBA9876543210n,
  0xFEDCBA9876543210n,
  'The result of (0x0n - -0xFEDCBA9876543210n) is 0xFEDCBA9876543210n'
);

assert.sameValue(
  -0x1n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876543211n,
  'The result of (-0x1n - 0xFEDCBA9876543210n) is -0xFEDCBA9876543211n'
);

assert.sameValue(
  -0x1n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9876543210n,
  'The result of (-0x1n - 0xFEDCBA987654320Fn) is -0xFEDCBA9876543210n'
);

assert.sameValue(
  -0x1n - 0xFEDCBA98n,
  -0xFEDCBA99n,
  'The result of (-0x1n - 0xFEDCBA98n) is -0xFEDCBA99n'
);

assert.sameValue(
  -0x1n - 0xFEDCBA97n,
  -0xFEDCBA98n,
  'The result of (-0x1n - 0xFEDCBA97n) is -0xFEDCBA98n'
);

assert.sameValue(-0x1n - 0x1234n, -0x1235n, 'The result of (-0x1n - 0x1234n) is -0x1235n');
assert.sameValue(-0x1n - 0x3n, -0x4n, 'The result of (-0x1n - 0x3n) is -0x4n');
assert.sameValue(-0x1n - 0x2n, -0x3n, 'The result of (-0x1n - 0x2n) is -0x3n');
assert.sameValue(-0x1n - 0x1n, -0x2n, 'The result of (-0x1n - 0x1n) is -0x2n');
assert.sameValue(-0x1n - 0x0n, -0x1n, 'The result of (-0x1n - 0x0n) is -0x1n');
assert.sameValue(-0x1n - -0x1n, 0x0n, 'The result of (-0x1n - -0x1n) is 0x0n');
assert.sameValue(-0x1n - -0x2n, 0x1n, 'The result of (-0x1n - -0x2n) is 0x1n');
assert.sameValue(-0x1n - -0x3n, 0x2n, 'The result of (-0x1n - -0x3n) is 0x2n');
assert.sameValue(-0x1n - -0x1234n, 0x1233n, 'The result of (-0x1n - -0x1234n) is 0x1233n');

assert.sameValue(
  -0x1n - -0xFEDCBA97n,
  0xFEDCBA96n,
  'The result of (-0x1n - -0xFEDCBA97n) is 0xFEDCBA96n'
);

assert.sameValue(
  -0x1n - -0xFEDCBA98n,
  0xFEDCBA97n,
  'The result of (-0x1n - -0xFEDCBA98n) is 0xFEDCBA97n'
);

assert.sameValue(
  -0x1n - -0xFEDCBA987654320Fn,
  0xFEDCBA987654320En,
  'The result of (-0x1n - -0xFEDCBA987654320Fn) is 0xFEDCBA987654320En'
);

assert.sameValue(
  -0x1n - -0xFEDCBA9876543210n,
  0xFEDCBA987654320Fn,
  'The result of (-0x1n - -0xFEDCBA9876543210n) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  -0x2n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876543212n,
  'The result of (-0x2n - 0xFEDCBA9876543210n) is -0xFEDCBA9876543212n'
);

assert.sameValue(
  -0x2n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9876543211n,
  'The result of (-0x2n - 0xFEDCBA987654320Fn) is -0xFEDCBA9876543211n'
);

assert.sameValue(
  -0x2n - 0xFEDCBA98n,
  -0xFEDCBA9An,
  'The result of (-0x2n - 0xFEDCBA98n) is -0xFEDCBA9An'
);

assert.sameValue(
  -0x2n - 0xFEDCBA97n,
  -0xFEDCBA99n,
  'The result of (-0x2n - 0xFEDCBA97n) is -0xFEDCBA99n'
);

assert.sameValue(-0x2n - 0x1234n, -0x1236n, 'The result of (-0x2n - 0x1234n) is -0x1236n');
assert.sameValue(-0x2n - 0x3n, -0x5n, 'The result of (-0x2n - 0x3n) is -0x5n');
assert.sameValue(-0x2n - 0x2n, -0x4n, 'The result of (-0x2n - 0x2n) is -0x4n');
assert.sameValue(-0x2n - 0x1n, -0x3n, 'The result of (-0x2n - 0x1n) is -0x3n');
assert.sameValue(-0x2n - 0x0n, -0x2n, 'The result of (-0x2n - 0x0n) is -0x2n');
assert.sameValue(-0x2n - -0x1n, -0x1n, 'The result of (-0x2n - -0x1n) is -0x1n');
assert.sameValue(-0x2n - -0x2n, 0x0n, 'The result of (-0x2n - -0x2n) is 0x0n');
assert.sameValue(-0x2n - -0x3n, 0x1n, 'The result of (-0x2n - -0x3n) is 0x1n');
assert.sameValue(-0x2n - -0x1234n, 0x1232n, 'The result of (-0x2n - -0x1234n) is 0x1232n');

assert.sameValue(
  -0x2n - -0xFEDCBA97n,
  0xFEDCBA95n,
  'The result of (-0x2n - -0xFEDCBA97n) is 0xFEDCBA95n'
);

assert.sameValue(
  -0x2n - -0xFEDCBA98n,
  0xFEDCBA96n,
  'The result of (-0x2n - -0xFEDCBA98n) is 0xFEDCBA96n'
);

assert.sameValue(
  -0x2n - -0xFEDCBA987654320Fn,
  0xFEDCBA987654320Dn,
  'The result of (-0x2n - -0xFEDCBA987654320Fn) is 0xFEDCBA987654320Dn'
);

assert.sameValue(
  -0x2n - -0xFEDCBA9876543210n,
  0xFEDCBA987654320En,
  'The result of (-0x2n - -0xFEDCBA9876543210n) is 0xFEDCBA987654320En'
);

assert.sameValue(
  -0x3n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876543213n,
  'The result of (-0x3n - 0xFEDCBA9876543210n) is -0xFEDCBA9876543213n'
);

assert.sameValue(
  -0x3n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9876543212n,
  'The result of (-0x3n - 0xFEDCBA987654320Fn) is -0xFEDCBA9876543212n'
);

assert.sameValue(
  -0x3n - 0xFEDCBA98n,
  -0xFEDCBA9Bn,
  'The result of (-0x3n - 0xFEDCBA98n) is -0xFEDCBA9Bn'
);

assert.sameValue(
  -0x3n - 0xFEDCBA97n,
  -0xFEDCBA9An,
  'The result of (-0x3n - 0xFEDCBA97n) is -0xFEDCBA9An'
);

assert.sameValue(-0x3n - 0x1234n, -0x1237n, 'The result of (-0x3n - 0x1234n) is -0x1237n');
assert.sameValue(-0x3n - 0x3n, -0x6n, 'The result of (-0x3n - 0x3n) is -0x6n');
assert.sameValue(-0x3n - 0x2n, -0x5n, 'The result of (-0x3n - 0x2n) is -0x5n');
assert.sameValue(-0x3n - 0x1n, -0x4n, 'The result of (-0x3n - 0x1n) is -0x4n');
assert.sameValue(-0x3n - 0x0n, -0x3n, 'The result of (-0x3n - 0x0n) is -0x3n');
assert.sameValue(-0x3n - -0x1n, -0x2n, 'The result of (-0x3n - -0x1n) is -0x2n');
assert.sameValue(-0x3n - -0x2n, -0x1n, 'The result of (-0x3n - -0x2n) is -0x1n');
assert.sameValue(-0x3n - -0x3n, 0x0n, 'The result of (-0x3n - -0x3n) is 0x0n');
assert.sameValue(-0x3n - -0x1234n, 0x1231n, 'The result of (-0x3n - -0x1234n) is 0x1231n');

assert.sameValue(
  -0x3n - -0xFEDCBA97n,
  0xFEDCBA94n,
  'The result of (-0x3n - -0xFEDCBA97n) is 0xFEDCBA94n'
);

assert.sameValue(
  -0x3n - -0xFEDCBA98n,
  0xFEDCBA95n,
  'The result of (-0x3n - -0xFEDCBA98n) is 0xFEDCBA95n'
);

assert.sameValue(
  -0x3n - -0xFEDCBA987654320Fn,
  0xFEDCBA987654320Cn,
  'The result of (-0x3n - -0xFEDCBA987654320Fn) is 0xFEDCBA987654320Cn'
);

assert.sameValue(
  -0x3n - -0xFEDCBA9876543210n,
  0xFEDCBA987654320Dn,
  'The result of (-0x3n - -0xFEDCBA9876543210n) is 0xFEDCBA987654320Dn'
);

assert.sameValue(
  -0x1234n - 0xFEDCBA9876543210n,
  -0xFEDCBA9876544444n,
  'The result of (-0x1234n - 0xFEDCBA9876543210n) is -0xFEDCBA9876544444n'
);

assert.sameValue(
  -0x1234n - 0xFEDCBA987654320Fn,
  -0xFEDCBA9876544443n,
  'The result of (-0x1234n - 0xFEDCBA987654320Fn) is -0xFEDCBA9876544443n'
);

assert.sameValue(
  -0x1234n - 0xFEDCBA98n,
  -0xFEDCCCCCn,
  'The result of (-0x1234n - 0xFEDCBA98n) is -0xFEDCCCCCn'
);

assert.sameValue(
  -0x1234n - 0xFEDCBA97n,
  -0xFEDCCCCBn,
  'The result of (-0x1234n - 0xFEDCBA97n) is -0xFEDCCCCBn'
);

assert.sameValue(-0x1234n - 0x1234n, -0x2468n, 'The result of (-0x1234n - 0x1234n) is -0x2468n');
assert.sameValue(-0x1234n - 0x3n, -0x1237n, 'The result of (-0x1234n - 0x3n) is -0x1237n');
assert.sameValue(-0x1234n - 0x2n, -0x1236n, 'The result of (-0x1234n - 0x2n) is -0x1236n');
assert.sameValue(-0x1234n - 0x1n, -0x1235n, 'The result of (-0x1234n - 0x1n) is -0x1235n');
assert.sameValue(-0x1234n - 0x0n, -0x1234n, 'The result of (-0x1234n - 0x0n) is -0x1234n');
assert.sameValue(-0x1234n - -0x1n, -0x1233n, 'The result of (-0x1234n - -0x1n) is -0x1233n');
assert.sameValue(-0x1234n - -0x2n, -0x1232n, 'The result of (-0x1234n - -0x2n) is -0x1232n');
assert.sameValue(-0x1234n - -0x3n, -0x1231n, 'The result of (-0x1234n - -0x3n) is -0x1231n');
assert.sameValue(-0x1234n - -0x1234n, 0x0n, 'The result of (-0x1234n - -0x1234n) is 0x0n');

assert.sameValue(
  -0x1234n - -0xFEDCBA97n,
  0xFEDCA863n,
  'The result of (-0x1234n - -0xFEDCBA97n) is 0xFEDCA863n'
);

assert.sameValue(
  -0x1234n - -0xFEDCBA98n,
  0xFEDCA864n,
  'The result of (-0x1234n - -0xFEDCBA98n) is 0xFEDCA864n'
);

assert.sameValue(
  -0x1234n - -0xFEDCBA987654320Fn,
  0xFEDCBA9876541FDBn,
  'The result of (-0x1234n - -0xFEDCBA987654320Fn) is 0xFEDCBA9876541FDBn'
);

assert.sameValue(
  -0x1234n - -0xFEDCBA9876543210n,
  0xFEDCBA9876541FDCn,
  'The result of (-0x1234n - -0xFEDCBA9876543210n) is 0xFEDCBA9876541FDCn'
);

assert.sameValue(
  -0xFEDCBA97n - 0xFEDCBA9876543210n,
  -0xFEDCBA997530ECA7n,
  'The result of (-0xFEDCBA97n - 0xFEDCBA9876543210n) is -0xFEDCBA997530ECA7n'
);

assert.sameValue(
  -0xFEDCBA97n - 0xFEDCBA987654320Fn,
  -0xFEDCBA997530ECA6n,
  'The result of (-0xFEDCBA97n - 0xFEDCBA987654320Fn) is -0xFEDCBA997530ECA6n'
);

assert.sameValue(
  -0xFEDCBA97n - 0xFEDCBA98n,
  -0x1FDB9752Fn,
  'The result of (-0xFEDCBA97n - 0xFEDCBA98n) is -0x1FDB9752Fn'
);

assert.sameValue(
  -0xFEDCBA97n - 0xFEDCBA97n,
  -0x1FDB9752En,
  'The result of (-0xFEDCBA97n - 0xFEDCBA97n) is -0x1FDB9752En'
);

assert.sameValue(
  -0xFEDCBA97n - 0x1234n,
  -0xFEDCCCCBn,
  'The result of (-0xFEDCBA97n - 0x1234n) is -0xFEDCCCCBn'
);

assert.sameValue(
  -0xFEDCBA97n - 0x3n,
  -0xFEDCBA9An,
  'The result of (-0xFEDCBA97n - 0x3n) is -0xFEDCBA9An'
);

assert.sameValue(
  -0xFEDCBA97n - 0x2n,
  -0xFEDCBA99n,
  'The result of (-0xFEDCBA97n - 0x2n) is -0xFEDCBA99n'
);

assert.sameValue(
  -0xFEDCBA97n - 0x1n,
  -0xFEDCBA98n,
  'The result of (-0xFEDCBA97n - 0x1n) is -0xFEDCBA98n'
);

assert.sameValue(
  -0xFEDCBA97n - 0x0n,
  -0xFEDCBA97n,
  'The result of (-0xFEDCBA97n - 0x0n) is -0xFEDCBA97n'
);

assert.sameValue(
  -0xFEDCBA97n - -0x1n,
  -0xFEDCBA96n,
  'The result of (-0xFEDCBA97n - -0x1n) is -0xFEDCBA96n'
);

assert.sameValue(
  -0xFEDCBA97n - -0x2n,
  -0xFEDCBA95n,
  'The result of (-0xFEDCBA97n - -0x2n) is -0xFEDCBA95n'
);

assert.sameValue(
  -0xFEDCBA97n - -0x3n,
  -0xFEDCBA94n,
  'The result of (-0xFEDCBA97n - -0x3n) is -0xFEDCBA94n'
);

assert.sameValue(
  -0xFEDCBA97n - -0x1234n,
  -0xFEDCA863n,
  'The result of (-0xFEDCBA97n - -0x1234n) is -0xFEDCA863n'
);

assert.sameValue(
  -0xFEDCBA97n - -0xFEDCBA97n,
  0x0n,
  'The result of (-0xFEDCBA97n - -0xFEDCBA97n) is 0x0n'
);

assert.sameValue(
  -0xFEDCBA97n - -0xFEDCBA98n,
  0x1n,
  'The result of (-0xFEDCBA97n - -0xFEDCBA98n) is 0x1n'
);

assert.sameValue(
  -0xFEDCBA97n - -0xFEDCBA987654320Fn,
  0xFEDCBA9777777778n,
  'The result of (-0xFEDCBA97n - -0xFEDCBA987654320Fn) is 0xFEDCBA9777777778n'
);

assert.sameValue(
  -0xFEDCBA97n - -0xFEDCBA9876543210n,
  0xFEDCBA9777777779n,
  'The result of (-0xFEDCBA97n - -0xFEDCBA9876543210n) is 0xFEDCBA9777777779n'
);

assert.sameValue(
  -0xFEDCBA98n - 0xFEDCBA9876543210n,
  -0xFEDCBA997530ECA8n,
  'The result of (-0xFEDCBA98n - 0xFEDCBA9876543210n) is -0xFEDCBA997530ECA8n'
);

assert.sameValue(
  -0xFEDCBA98n - 0xFEDCBA987654320Fn,
  -0xFEDCBA997530ECA7n,
  'The result of (-0xFEDCBA98n - 0xFEDCBA987654320Fn) is -0xFEDCBA997530ECA7n'
);

assert.sameValue(
  -0xFEDCBA98n - 0xFEDCBA98n,
  -0x1FDB97530n,
  'The result of (-0xFEDCBA98n - 0xFEDCBA98n) is -0x1FDB97530n'
);

assert.sameValue(
  -0xFEDCBA98n - 0xFEDCBA97n,
  -0x1FDB9752Fn,
  'The result of (-0xFEDCBA98n - 0xFEDCBA97n) is -0x1FDB9752Fn'
);

assert.sameValue(
  -0xFEDCBA98n - 0x1234n,
  -0xFEDCCCCCn,
  'The result of (-0xFEDCBA98n - 0x1234n) is -0xFEDCCCCCn'
);

assert.sameValue(
  -0xFEDCBA98n - 0x3n,
  -0xFEDCBA9Bn,
  'The result of (-0xFEDCBA98n - 0x3n) is -0xFEDCBA9Bn'
);

assert.sameValue(
  -0xFEDCBA98n - 0x2n,
  -0xFEDCBA9An,
  'The result of (-0xFEDCBA98n - 0x2n) is -0xFEDCBA9An'
);

assert.sameValue(
  -0xFEDCBA98n - 0x1n,
  -0xFEDCBA99n,
  'The result of (-0xFEDCBA98n - 0x1n) is -0xFEDCBA99n'
);

assert.sameValue(
  -0xFEDCBA98n - 0x0n,
  -0xFEDCBA98n,
  'The result of (-0xFEDCBA98n - 0x0n) is -0xFEDCBA98n'
);

assert.sameValue(
  -0xFEDCBA98n - -0x1n,
  -0xFEDCBA97n,
  'The result of (-0xFEDCBA98n - -0x1n) is -0xFEDCBA97n'
);

assert.sameValue(
  -0xFEDCBA98n - -0x2n,
  -0xFEDCBA96n,
  'The result of (-0xFEDCBA98n - -0x2n) is -0xFEDCBA96n'
);

assert.sameValue(
  -0xFEDCBA98n - -0x3n,
  -0xFEDCBA95n,
  'The result of (-0xFEDCBA98n - -0x3n) is -0xFEDCBA95n'
);

assert.sameValue(
  -0xFEDCBA98n - -0x1234n,
  -0xFEDCA864n,
  'The result of (-0xFEDCBA98n - -0x1234n) is -0xFEDCA864n'
);

assert.sameValue(
  -0xFEDCBA98n - -0xFEDCBA97n,
  -0x1n,
  'The result of (-0xFEDCBA98n - -0xFEDCBA97n) is -0x1n'
);

assert.sameValue(
  -0xFEDCBA98n - -0xFEDCBA98n,
  0x0n,
  'The result of (-0xFEDCBA98n - -0xFEDCBA98n) is 0x0n'
);

assert.sameValue(
  -0xFEDCBA98n - -0xFEDCBA987654320Fn,
  0xFEDCBA9777777777n,
  'The result of (-0xFEDCBA98n - -0xFEDCBA987654320Fn) is 0xFEDCBA9777777777n'
);

assert.sameValue(
  -0xFEDCBA98n - -0xFEDCBA9876543210n,
  0xFEDCBA9777777778n,
  'The result of (-0xFEDCBA98n - -0xFEDCBA9876543210n) is 0xFEDCBA9777777778n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0xFEDCBA9876543210n,
  -0x1FDB97530ECA8641Fn,
  'The result of (-0xFEDCBA987654320Fn - 0xFEDCBA9876543210n) is -0x1FDB97530ECA8641Fn'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0xFEDCBA987654320Fn,
  -0x1FDB97530ECA8641En,
  'The result of (-0xFEDCBA987654320Fn - 0xFEDCBA987654320Fn) is -0x1FDB97530ECA8641En'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0xFEDCBA98n,
  -0xFEDCBA997530ECA7n,
  'The result of (-0xFEDCBA987654320Fn - 0xFEDCBA98n) is -0xFEDCBA997530ECA7n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0xFEDCBA97n,
  -0xFEDCBA997530ECA6n,
  'The result of (-0xFEDCBA987654320Fn - 0xFEDCBA97n) is -0xFEDCBA997530ECA6n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0x1234n,
  -0xFEDCBA9876544443n,
  'The result of (-0xFEDCBA987654320Fn - 0x1234n) is -0xFEDCBA9876544443n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0x3n,
  -0xFEDCBA9876543212n,
  'The result of (-0xFEDCBA987654320Fn - 0x3n) is -0xFEDCBA9876543212n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0x2n,
  -0xFEDCBA9876543211n,
  'The result of (-0xFEDCBA987654320Fn - 0x2n) is -0xFEDCBA9876543211n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0x1n,
  -0xFEDCBA9876543210n,
  'The result of (-0xFEDCBA987654320Fn - 0x1n) is -0xFEDCBA9876543210n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - 0x0n,
  -0xFEDCBA987654320Fn,
  'The result of (-0xFEDCBA987654320Fn - 0x0n) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0x1n,
  -0xFEDCBA987654320En,
  'The result of (-0xFEDCBA987654320Fn - -0x1n) is -0xFEDCBA987654320En'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0x2n,
  -0xFEDCBA987654320Dn,
  'The result of (-0xFEDCBA987654320Fn - -0x2n) is -0xFEDCBA987654320Dn'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0x3n,
  -0xFEDCBA987654320Cn,
  'The result of (-0xFEDCBA987654320Fn - -0x3n) is -0xFEDCBA987654320Cn'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0x1234n,
  -0xFEDCBA9876541FDBn,
  'The result of (-0xFEDCBA987654320Fn - -0x1234n) is -0xFEDCBA9876541FDBn'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0xFEDCBA97n,
  -0xFEDCBA9777777778n,
  'The result of (-0xFEDCBA987654320Fn - -0xFEDCBA97n) is -0xFEDCBA9777777778n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0xFEDCBA98n,
  -0xFEDCBA9777777777n,
  'The result of (-0xFEDCBA987654320Fn - -0xFEDCBA98n) is -0xFEDCBA9777777777n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0xFEDCBA987654320Fn,
  0x0n,
  'The result of (-0xFEDCBA987654320Fn - -0xFEDCBA987654320Fn) is 0x0n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn - -0xFEDCBA9876543210n,
  0x1n,
  'The result of (-0xFEDCBA987654320Fn - -0xFEDCBA9876543210n) is 0x1n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0xFEDCBA9876543210n,
  -0x1FDB97530ECA86420n,
  'The result of (-0xFEDCBA9876543210n - 0xFEDCBA9876543210n) is -0x1FDB97530ECA86420n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0xFEDCBA987654320Fn,
  -0x1FDB97530ECA8641Fn,
  'The result of (-0xFEDCBA9876543210n - 0xFEDCBA987654320Fn) is -0x1FDB97530ECA8641Fn'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0xFEDCBA98n,
  -0xFEDCBA997530ECA8n,
  'The result of (-0xFEDCBA9876543210n - 0xFEDCBA98n) is -0xFEDCBA997530ECA8n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0xFEDCBA97n,
  -0xFEDCBA997530ECA7n,
  'The result of (-0xFEDCBA9876543210n - 0xFEDCBA97n) is -0xFEDCBA997530ECA7n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0x1234n,
  -0xFEDCBA9876544444n,
  'The result of (-0xFEDCBA9876543210n - 0x1234n) is -0xFEDCBA9876544444n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0x3n,
  -0xFEDCBA9876543213n,
  'The result of (-0xFEDCBA9876543210n - 0x3n) is -0xFEDCBA9876543213n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0x2n,
  -0xFEDCBA9876543212n,
  'The result of (-0xFEDCBA9876543210n - 0x2n) is -0xFEDCBA9876543212n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0x1n,
  -0xFEDCBA9876543211n,
  'The result of (-0xFEDCBA9876543210n - 0x1n) is -0xFEDCBA9876543211n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - 0x0n,
  -0xFEDCBA9876543210n,
  'The result of (-0xFEDCBA9876543210n - 0x0n) is -0xFEDCBA9876543210n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0x1n,
  -0xFEDCBA987654320Fn,
  'The result of (-0xFEDCBA9876543210n - -0x1n) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0x2n,
  -0xFEDCBA987654320En,
  'The result of (-0xFEDCBA9876543210n - -0x2n) is -0xFEDCBA987654320En'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0x3n,
  -0xFEDCBA987654320Dn,
  'The result of (-0xFEDCBA9876543210n - -0x3n) is -0xFEDCBA987654320Dn'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0x1234n,
  -0xFEDCBA9876541FDCn,
  'The result of (-0xFEDCBA9876543210n - -0x1234n) is -0xFEDCBA9876541FDCn'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0xFEDCBA97n,
  -0xFEDCBA9777777779n,
  'The result of (-0xFEDCBA9876543210n - -0xFEDCBA97n) is -0xFEDCBA9777777779n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0xFEDCBA98n,
  -0xFEDCBA9777777778n,
  'The result of (-0xFEDCBA9876543210n - -0xFEDCBA98n) is -0xFEDCBA9777777778n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0xFEDCBA987654320Fn,
  -0x1n,
  'The result of (-0xFEDCBA9876543210n - -0xFEDCBA987654320Fn) is -0x1n'
);

assert.sameValue(
  -0xFEDCBA9876543210n - -0xFEDCBA9876543210n,
  0x0n,
  'The result of (-0xFEDCBA9876543210n - -0xFEDCBA9876543210n) is 0x0n'
);
