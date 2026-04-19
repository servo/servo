// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-multiplicative-operators-runtime-semantics-evaluation
description: BigInt multiplication arithmetic
features: [BigInt]
---*/
assert.sameValue(
  0xFEDCBA9876543210n * 0xFEDCBA9876543210n,
  0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n,
  'The result of (0xFEDCBA9876543210n * 0xFEDCBA9876543210n) is 0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0xFEDCBA987654320Fn,
  0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n,
  'The result of (0xFEDCBA9876543210n * 0xFEDCBA987654320Fn) is 0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0xFEDCBA98n,
  0xFDBAC097530ECA86541D5980n,
  'The result of (0xFEDCBA9876543210n * 0xFEDCBA98n) is 0xFDBAC097530ECA86541D5980n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0xFEDCBA97n,
  0xFDBAC09654320FEDDDC92770n,
  'The result of (0xFEDCBA9876543210n * 0xFEDCBA97n) is 0xFDBAC09654320FEDDDC92770n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0x1234n,
  0x121F49F49F49F49F4B40n,
  'The result of (0xFEDCBA9876543210n * 0x1234n) is 0x121F49F49F49F49F4B40n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0x3n,
  0x2FC962FC962FC9630n,
  'The result of (0xFEDCBA9876543210n * 0x3n) is 0x2FC962FC962FC9630n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0x2n,
  0x1FDB97530ECA86420n,
  'The result of (0xFEDCBA9876543210n * 0x2n) is 0x1FDB97530ECA86420n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0x1n,
  0xFEDCBA9876543210n,
  'The result of (0xFEDCBA9876543210n * 0x1n) is 0xFEDCBA9876543210n'
);

assert.sameValue(
  0xFEDCBA9876543210n * 0x0n,
  0x0n,
  'The result of (0xFEDCBA9876543210n * 0x0n) is 0x0n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0x1n,
  -0xFEDCBA9876543210n,
  'The result of (0xFEDCBA9876543210n * -0x1n) is -0xFEDCBA9876543210n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0x2n,
  -0x1FDB97530ECA86420n,
  'The result of (0xFEDCBA9876543210n * -0x2n) is -0x1FDB97530ECA86420n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0x3n,
  -0x2FC962FC962FC9630n,
  'The result of (0xFEDCBA9876543210n * -0x3n) is -0x2FC962FC962FC9630n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0x1234n,
  -0x121F49F49F49F49F4B40n,
  'The result of (0xFEDCBA9876543210n * -0x1234n) is -0x121F49F49F49F49F4B40n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0xFEDCBA97n,
  -0xFDBAC09654320FEDDDC92770n,
  'The result of (0xFEDCBA9876543210n * -0xFEDCBA97n) is -0xFDBAC09654320FEDDDC92770n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0xFEDCBA98n,
  -0xFDBAC097530ECA86541D5980n,
  'The result of (0xFEDCBA9876543210n * -0xFEDCBA98n) is -0xFDBAC097530ECA86541D5980n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0xFEDCBA987654320Fn,
  -0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n,
  'The result of (0xFEDCBA9876543210n * -0xFEDCBA987654320Fn) is -0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n'
);

assert.sameValue(
  0xFEDCBA9876543210n * -0xFEDCBA9876543210n,
  -0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n,
  'The result of (0xFEDCBA9876543210n * -0xFEDCBA9876543210n) is -0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0xFEDCBA987654320Fn,
  0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n,
  'The result of (0xFEDCBA987654320Fn * 0xFEDCBA987654320Fn) is 0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0xFEDCBA98n,
  0xFDBAC097530ECA8555409EE8n,
  'The result of (0xFEDCBA987654320Fn * 0xFEDCBA98n) is 0xFDBAC097530ECA8555409EE8n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0xFEDCBA97n,
  0xFDBAC09654320FECDEEC6CD9n,
  'The result of (0xFEDCBA987654320Fn * 0xFEDCBA97n) is 0xFDBAC09654320FECDEEC6CD9n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0x1234n,
  0x121F49F49F49F49F390Cn,
  'The result of (0xFEDCBA987654320Fn * 0x1234n) is 0x121F49F49F49F49F390Cn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0x3n,
  0x2FC962FC962FC962Dn,
  'The result of (0xFEDCBA987654320Fn * 0x3n) is 0x2FC962FC962FC962Dn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0x2n,
  0x1FDB97530ECA8641En,
  'The result of (0xFEDCBA987654320Fn * 0x2n) is 0x1FDB97530ECA8641En'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0x1n,
  0xFEDCBA987654320Fn,
  'The result of (0xFEDCBA987654320Fn * 0x1n) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * 0x0n,
  0x0n,
  'The result of (0xFEDCBA987654320Fn * 0x0n) is 0x0n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0x1n,
  -0xFEDCBA987654320Fn,
  'The result of (0xFEDCBA987654320Fn * -0x1n) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0x2n,
  -0x1FDB97530ECA8641En,
  'The result of (0xFEDCBA987654320Fn * -0x2n) is -0x1FDB97530ECA8641En'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0x3n,
  -0x2FC962FC962FC962Dn,
  'The result of (0xFEDCBA987654320Fn * -0x3n) is -0x2FC962FC962FC962Dn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0x1234n,
  -0x121F49F49F49F49F390Cn,
  'The result of (0xFEDCBA987654320Fn * -0x1234n) is -0x121F49F49F49F49F390Cn'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0xFEDCBA97n,
  -0xFDBAC09654320FECDEEC6CD9n,
  'The result of (0xFEDCBA987654320Fn * -0xFEDCBA97n) is -0xFDBAC09654320FECDEEC6CD9n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0xFEDCBA98n,
  -0xFDBAC097530ECA8555409EE8n,
  'The result of (0xFEDCBA987654320Fn * -0xFEDCBA98n) is -0xFDBAC097530ECA8555409EE8n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0xFEDCBA987654320Fn,
  -0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n,
  'The result of (0xFEDCBA987654320Fn * -0xFEDCBA987654320Fn) is -0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n'
);

assert.sameValue(
  0xFEDCBA987654320Fn * -0xFEDCBA9876543210n,
  -0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n,
  'The result of (0xFEDCBA987654320Fn * -0xFEDCBA9876543210n) is -0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n'
);

assert.sameValue(
  0xFEDCBA98n * 0xFEDCBA98n,
  0xFDBAC096DD413A40n,
  'The result of (0xFEDCBA98n * 0xFEDCBA98n) is 0xFDBAC096DD413A40n'
);

assert.sameValue(
  0xFEDCBA98n * 0xFEDCBA97n,
  0xFDBAC095DE647FA8n,
  'The result of (0xFEDCBA98n * 0xFEDCBA97n) is 0xFDBAC095DE647FA8n'
);

assert.sameValue(
  0xFEDCBA98n * 0x1234n,
  0x121F49F496E0n,
  'The result of (0xFEDCBA98n * 0x1234n) is 0x121F49F496E0n'
);

assert.sameValue(
  0xFEDCBA98n * 0x3n,
  0x2FC962FC8n,
  'The result of (0xFEDCBA98n * 0x3n) is 0x2FC962FC8n'
);

assert.sameValue(
  0xFEDCBA98n * 0x2n,
  0x1FDB97530n,
  'The result of (0xFEDCBA98n * 0x2n) is 0x1FDB97530n'
);

assert.sameValue(
  0xFEDCBA98n * 0x1n,
  0xFEDCBA98n,
  'The result of (0xFEDCBA98n * 0x1n) is 0xFEDCBA98n'
);

assert.sameValue(0xFEDCBA98n * 0x0n, 0x0n, 'The result of (0xFEDCBA98n * 0x0n) is 0x0n');

assert.sameValue(
  0xFEDCBA98n * -0x1n,
  -0xFEDCBA98n,
  'The result of (0xFEDCBA98n * -0x1n) is -0xFEDCBA98n'
);

assert.sameValue(
  0xFEDCBA98n * -0x2n,
  -0x1FDB97530n,
  'The result of (0xFEDCBA98n * -0x2n) is -0x1FDB97530n'
);

assert.sameValue(
  0xFEDCBA98n * -0x3n,
  -0x2FC962FC8n,
  'The result of (0xFEDCBA98n * -0x3n) is -0x2FC962FC8n'
);

assert.sameValue(
  0xFEDCBA98n * -0x1234n,
  -0x121F49F496E0n,
  'The result of (0xFEDCBA98n * -0x1234n) is -0x121F49F496E0n'
);

assert.sameValue(
  0xFEDCBA98n * -0xFEDCBA97n,
  -0xFDBAC095DE647FA8n,
  'The result of (0xFEDCBA98n * -0xFEDCBA97n) is -0xFDBAC095DE647FA8n'
);

assert.sameValue(
  0xFEDCBA98n * -0xFEDCBA98n,
  -0xFDBAC096DD413A40n,
  'The result of (0xFEDCBA98n * -0xFEDCBA98n) is -0xFDBAC096DD413A40n'
);

assert.sameValue(
  0xFEDCBA98n * -0xFEDCBA987654320Fn,
  -0xFDBAC097530ECA8555409EE8n,
  'The result of (0xFEDCBA98n * -0xFEDCBA987654320Fn) is -0xFDBAC097530ECA8555409EE8n'
);

assert.sameValue(
  0xFEDCBA98n * -0xFEDCBA9876543210n,
  -0xFDBAC097530ECA86541D5980n,
  'The result of (0xFEDCBA98n * -0xFEDCBA9876543210n) is -0xFDBAC097530ECA86541D5980n'
);

assert.sameValue(
  0xFEDCBA97n * 0xFEDCBA97n,
  0xFDBAC094DF87C511n,
  'The result of (0xFEDCBA97n * 0xFEDCBA97n) is 0xFDBAC094DF87C511n'
);

assert.sameValue(
  0xFEDCBA97n * 0x1234n,
  0x121F49F484ACn,
  'The result of (0xFEDCBA97n * 0x1234n) is 0x121F49F484ACn'
);

assert.sameValue(
  0xFEDCBA97n * 0x3n,
  0x2FC962FC5n,
  'The result of (0xFEDCBA97n * 0x3n) is 0x2FC962FC5n'
);

assert.sameValue(
  0xFEDCBA97n * 0x2n,
  0x1FDB9752En,
  'The result of (0xFEDCBA97n * 0x2n) is 0x1FDB9752En'
);

assert.sameValue(
  0xFEDCBA97n * 0x1n,
  0xFEDCBA97n,
  'The result of (0xFEDCBA97n * 0x1n) is 0xFEDCBA97n'
);

assert.sameValue(0xFEDCBA97n * 0x0n, 0x0n, 'The result of (0xFEDCBA97n * 0x0n) is 0x0n');

assert.sameValue(
  0xFEDCBA97n * -0x1n,
  -0xFEDCBA97n,
  'The result of (0xFEDCBA97n * -0x1n) is -0xFEDCBA97n'
);

assert.sameValue(
  0xFEDCBA97n * -0x2n,
  -0x1FDB9752En,
  'The result of (0xFEDCBA97n * -0x2n) is -0x1FDB9752En'
);

assert.sameValue(
  0xFEDCBA97n * -0x3n,
  -0x2FC962FC5n,
  'The result of (0xFEDCBA97n * -0x3n) is -0x2FC962FC5n'
);

assert.sameValue(
  0xFEDCBA97n * -0x1234n,
  -0x121F49F484ACn,
  'The result of (0xFEDCBA97n * -0x1234n) is -0x121F49F484ACn'
);

assert.sameValue(
  0xFEDCBA97n * -0xFEDCBA97n,
  -0xFDBAC094DF87C511n,
  'The result of (0xFEDCBA97n * -0xFEDCBA97n) is -0xFDBAC094DF87C511n'
);

assert.sameValue(
  0xFEDCBA97n * -0xFEDCBA98n,
  -0xFDBAC095DE647FA8n,
  'The result of (0xFEDCBA97n * -0xFEDCBA98n) is -0xFDBAC095DE647FA8n'
);

assert.sameValue(
  0xFEDCBA97n * -0xFEDCBA987654320Fn,
  -0xFDBAC09654320FECDEEC6CD9n,
  'The result of (0xFEDCBA97n * -0xFEDCBA987654320Fn) is -0xFDBAC09654320FECDEEC6CD9n'
);

assert.sameValue(
  0xFEDCBA97n * -0xFEDCBA9876543210n,
  -0xFDBAC09654320FEDDDC92770n,
  'The result of (0xFEDCBA97n * -0xFEDCBA9876543210n) is -0xFDBAC09654320FEDDDC92770n'
);

assert.sameValue(0x1234n * 0x1234n, 0x14B5A90n, 'The result of (0x1234n * 0x1234n) is 0x14B5A90n');
assert.sameValue(0x1234n * 0x3n, 0x369Cn, 'The result of (0x1234n * 0x3n) is 0x369Cn');
assert.sameValue(0x1234n * 0x2n, 0x2468n, 'The result of (0x1234n * 0x2n) is 0x2468n');
assert.sameValue(0x1234n * 0x1n, 0x1234n, 'The result of (0x1234n * 0x1n) is 0x1234n');
assert.sameValue(0x1234n * 0x0n, 0x0n, 'The result of (0x1234n * 0x0n) is 0x0n');
assert.sameValue(0x1234n * -0x1n, -0x1234n, 'The result of (0x1234n * -0x1n) is -0x1234n');
assert.sameValue(0x1234n * -0x2n, -0x2468n, 'The result of (0x1234n * -0x2n) is -0x2468n');
assert.sameValue(0x1234n * -0x3n, -0x369Cn, 'The result of (0x1234n * -0x3n) is -0x369Cn');

assert.sameValue(
  0x1234n * -0x1234n,
  -0x14B5A90n,
  'The result of (0x1234n * -0x1234n) is -0x14B5A90n'
);

assert.sameValue(
  0x1234n * -0xFEDCBA97n,
  -0x121F49F484ACn,
  'The result of (0x1234n * -0xFEDCBA97n) is -0x121F49F484ACn'
);

assert.sameValue(
  0x1234n * -0xFEDCBA98n,
  -0x121F49F496E0n,
  'The result of (0x1234n * -0xFEDCBA98n) is -0x121F49F496E0n'
);

assert.sameValue(
  0x1234n * -0xFEDCBA987654320Fn,
  -0x121F49F49F49F49F390Cn,
  'The result of (0x1234n * -0xFEDCBA987654320Fn) is -0x121F49F49F49F49F390Cn'
);

assert.sameValue(
  0x1234n * -0xFEDCBA9876543210n,
  -0x121F49F49F49F49F4B40n,
  'The result of (0x1234n * -0xFEDCBA9876543210n) is -0x121F49F49F49F49F4B40n'
);

assert.sameValue(0x3n * 0x3n, 0x9n, 'The result of (0x3n * 0x3n) is 0x9n');
assert.sameValue(0x3n * 0x2n, 0x6n, 'The result of (0x3n * 0x2n) is 0x6n');
assert.sameValue(0x3n * 0x1n, 0x3n, 'The result of (0x3n * 0x1n) is 0x3n');
assert.sameValue(0x3n * 0x0n, 0x0n, 'The result of (0x3n * 0x0n) is 0x0n');
assert.sameValue(0x3n * -0x1n, -0x3n, 'The result of (0x3n * -0x1n) is -0x3n');
assert.sameValue(0x3n * -0x2n, -0x6n, 'The result of (0x3n * -0x2n) is -0x6n');
assert.sameValue(0x3n * -0x3n, -0x9n, 'The result of (0x3n * -0x3n) is -0x9n');
assert.sameValue(0x3n * -0x1234n, -0x369Cn, 'The result of (0x3n * -0x1234n) is -0x369Cn');

assert.sameValue(
  0x3n * -0xFEDCBA97n,
  -0x2FC962FC5n,
  'The result of (0x3n * -0xFEDCBA97n) is -0x2FC962FC5n'
);

assert.sameValue(
  0x3n * -0xFEDCBA98n,
  -0x2FC962FC8n,
  'The result of (0x3n * -0xFEDCBA98n) is -0x2FC962FC8n'
);

assert.sameValue(
  0x3n * -0xFEDCBA987654320Fn,
  -0x2FC962FC962FC962Dn,
  'The result of (0x3n * -0xFEDCBA987654320Fn) is -0x2FC962FC962FC962Dn'
);

assert.sameValue(
  0x3n * -0xFEDCBA9876543210n,
  -0x2FC962FC962FC9630n,
  'The result of (0x3n * -0xFEDCBA9876543210n) is -0x2FC962FC962FC9630n'
);

assert.sameValue(0x2n * 0x2n, 0x4n, 'The result of (0x2n * 0x2n) is 0x4n');
assert.sameValue(0x2n * 0x1n, 0x2n, 'The result of (0x2n * 0x1n) is 0x2n');
assert.sameValue(0x2n * 0x0n, 0x0n, 'The result of (0x2n * 0x0n) is 0x0n');
assert.sameValue(0x2n * -0x1n, -0x2n, 'The result of (0x2n * -0x1n) is -0x2n');
assert.sameValue(0x2n * -0x2n, -0x4n, 'The result of (0x2n * -0x2n) is -0x4n');
assert.sameValue(0x2n * -0x3n, -0x6n, 'The result of (0x2n * -0x3n) is -0x6n');
assert.sameValue(0x2n * -0x1234n, -0x2468n, 'The result of (0x2n * -0x1234n) is -0x2468n');

assert.sameValue(
  0x2n * -0xFEDCBA97n,
  -0x1FDB9752En,
  'The result of (0x2n * -0xFEDCBA97n) is -0x1FDB9752En'
);

assert.sameValue(
  0x2n * -0xFEDCBA98n,
  -0x1FDB97530n,
  'The result of (0x2n * -0xFEDCBA98n) is -0x1FDB97530n'
);

assert.sameValue(
  0x2n * -0xFEDCBA987654320Fn,
  -0x1FDB97530ECA8641En,
  'The result of (0x2n * -0xFEDCBA987654320Fn) is -0x1FDB97530ECA8641En'
);

assert.sameValue(
  0x2n * -0xFEDCBA9876543210n,
  -0x1FDB97530ECA86420n,
  'The result of (0x2n * -0xFEDCBA9876543210n) is -0x1FDB97530ECA86420n'
);

assert.sameValue(0x1n * 0x1n, 0x1n, 'The result of (0x1n * 0x1n) is 0x1n');
assert.sameValue(0x1n * 0x0n, 0x0n, 'The result of (0x1n * 0x0n) is 0x0n');
assert.sameValue(0x1n * -0x1n, -0x1n, 'The result of (0x1n * -0x1n) is -0x1n');
assert.sameValue(0x1n * -0x2n, -0x2n, 'The result of (0x1n * -0x2n) is -0x2n');
assert.sameValue(0x1n * -0x3n, -0x3n, 'The result of (0x1n * -0x3n) is -0x3n');
assert.sameValue(0x1n * -0x1234n, -0x1234n, 'The result of (0x1n * -0x1234n) is -0x1234n');

assert.sameValue(
  0x1n * -0xFEDCBA97n,
  -0xFEDCBA97n,
  'The result of (0x1n * -0xFEDCBA97n) is -0xFEDCBA97n'
);

assert.sameValue(
  0x1n * -0xFEDCBA98n,
  -0xFEDCBA98n,
  'The result of (0x1n * -0xFEDCBA98n) is -0xFEDCBA98n'
);

assert.sameValue(
  0x1n * -0xFEDCBA987654320Fn,
  -0xFEDCBA987654320Fn,
  'The result of (0x1n * -0xFEDCBA987654320Fn) is -0xFEDCBA987654320Fn'
);

assert.sameValue(
  0x1n * -0xFEDCBA9876543210n,
  -0xFEDCBA9876543210n,
  'The result of (0x1n * -0xFEDCBA9876543210n) is -0xFEDCBA9876543210n'
);

assert.sameValue(0x0n * 0x0n, 0x0n, 'The result of (0x0n * 0x0n) is 0x0n');
assert.sameValue(0x0n * -0x1n, 0x0n, 'The result of (0x0n * -0x1n) is 0x0n');
assert.sameValue(0x0n * -0x2n, 0x0n, 'The result of (0x0n * -0x2n) is 0x0n');
assert.sameValue(0x0n * -0x3n, 0x0n, 'The result of (0x0n * -0x3n) is 0x0n');
assert.sameValue(0x0n * -0x1234n, 0x0n, 'The result of (0x0n * -0x1234n) is 0x0n');
assert.sameValue(0x0n * -0xFEDCBA97n, 0x0n, 'The result of (0x0n * -0xFEDCBA97n) is 0x0n');
assert.sameValue(0x0n * -0xFEDCBA98n, 0x0n, 'The result of (0x0n * -0xFEDCBA98n) is 0x0n');

assert.sameValue(
  0x0n * -0xFEDCBA987654320Fn,
  0x0n,
  'The result of (0x0n * -0xFEDCBA987654320Fn) is 0x0n'
);

assert.sameValue(
  0x0n * -0xFEDCBA9876543210n,
  0x0n,
  'The result of (0x0n * -0xFEDCBA9876543210n) is 0x0n'
);

assert.sameValue(-0x1n * -0x1n, 0x1n, 'The result of (-0x1n * -0x1n) is 0x1n');
assert.sameValue(-0x1n * -0x2n, 0x2n, 'The result of (-0x1n * -0x2n) is 0x2n');
assert.sameValue(-0x1n * -0x3n, 0x3n, 'The result of (-0x1n * -0x3n) is 0x3n');
assert.sameValue(-0x1n * -0x1234n, 0x1234n, 'The result of (-0x1n * -0x1234n) is 0x1234n');

assert.sameValue(
  -0x1n * -0xFEDCBA97n,
  0xFEDCBA97n,
  'The result of (-0x1n * -0xFEDCBA97n) is 0xFEDCBA97n'
);

assert.sameValue(
  -0x1n * -0xFEDCBA98n,
  0xFEDCBA98n,
  'The result of (-0x1n * -0xFEDCBA98n) is 0xFEDCBA98n'
);

assert.sameValue(
  -0x1n * -0xFEDCBA987654320Fn,
  0xFEDCBA987654320Fn,
  'The result of (-0x1n * -0xFEDCBA987654320Fn) is 0xFEDCBA987654320Fn'
);

assert.sameValue(
  -0x1n * -0xFEDCBA9876543210n,
  0xFEDCBA9876543210n,
  'The result of (-0x1n * -0xFEDCBA9876543210n) is 0xFEDCBA9876543210n'
);

assert.sameValue(-0x2n * -0x2n, 0x4n, 'The result of (-0x2n * -0x2n) is 0x4n');
assert.sameValue(-0x2n * -0x3n, 0x6n, 'The result of (-0x2n * -0x3n) is 0x6n');
assert.sameValue(-0x2n * -0x1234n, 0x2468n, 'The result of (-0x2n * -0x1234n) is 0x2468n');

assert.sameValue(
  -0x2n * -0xFEDCBA97n,
  0x1FDB9752En,
  'The result of (-0x2n * -0xFEDCBA97n) is 0x1FDB9752En'
);

assert.sameValue(
  -0x2n * -0xFEDCBA98n,
  0x1FDB97530n,
  'The result of (-0x2n * -0xFEDCBA98n) is 0x1FDB97530n'
);

assert.sameValue(
  -0x2n * -0xFEDCBA987654320Fn,
  0x1FDB97530ECA8641En,
  'The result of (-0x2n * -0xFEDCBA987654320Fn) is 0x1FDB97530ECA8641En'
);

assert.sameValue(
  -0x2n * -0xFEDCBA9876543210n,
  0x1FDB97530ECA86420n,
  'The result of (-0x2n * -0xFEDCBA9876543210n) is 0x1FDB97530ECA86420n'
);

assert.sameValue(-0x3n * -0x3n, 0x9n, 'The result of (-0x3n * -0x3n) is 0x9n');
assert.sameValue(-0x3n * -0x1234n, 0x369Cn, 'The result of (-0x3n * -0x1234n) is 0x369Cn');

assert.sameValue(
  -0x3n * -0xFEDCBA97n,
  0x2FC962FC5n,
  'The result of (-0x3n * -0xFEDCBA97n) is 0x2FC962FC5n'
);

assert.sameValue(
  -0x3n * -0xFEDCBA98n,
  0x2FC962FC8n,
  'The result of (-0x3n * -0xFEDCBA98n) is 0x2FC962FC8n'
);

assert.sameValue(
  -0x3n * -0xFEDCBA987654320Fn,
  0x2FC962FC962FC962Dn,
  'The result of (-0x3n * -0xFEDCBA987654320Fn) is 0x2FC962FC962FC962Dn'
);

assert.sameValue(
  -0x3n * -0xFEDCBA9876543210n,
  0x2FC962FC962FC9630n,
  'The result of (-0x3n * -0xFEDCBA9876543210n) is 0x2FC962FC962FC9630n'
);

assert.sameValue(
  -0x1234n * -0x1234n,
  0x14B5A90n,
  'The result of (-0x1234n * -0x1234n) is 0x14B5A90n'
);

assert.sameValue(
  -0x1234n * -0xFEDCBA97n,
  0x121F49F484ACn,
  'The result of (-0x1234n * -0xFEDCBA97n) is 0x121F49F484ACn'
);

assert.sameValue(
  -0x1234n * -0xFEDCBA98n,
  0x121F49F496E0n,
  'The result of (-0x1234n * -0xFEDCBA98n) is 0x121F49F496E0n'
);

assert.sameValue(
  -0x1234n * -0xFEDCBA987654320Fn,
  0x121F49F49F49F49F390Cn,
  'The result of (-0x1234n * -0xFEDCBA987654320Fn) is 0x121F49F49F49F49F390Cn'
);

assert.sameValue(
  -0x1234n * -0xFEDCBA9876543210n,
  0x121F49F49F49F49F4B40n,
  'The result of (-0x1234n * -0xFEDCBA9876543210n) is 0x121F49F49F49F49F4B40n'
);

assert.sameValue(
  -0xFEDCBA97n * -0xFEDCBA97n,
  0xFDBAC094DF87C511n,
  'The result of (-0xFEDCBA97n * -0xFEDCBA97n) is 0xFDBAC094DF87C511n'
);

assert.sameValue(
  -0xFEDCBA97n * -0xFEDCBA98n,
  0xFDBAC095DE647FA8n,
  'The result of (-0xFEDCBA97n * -0xFEDCBA98n) is 0xFDBAC095DE647FA8n'
);

assert.sameValue(
  -0xFEDCBA97n * -0xFEDCBA987654320Fn,
  0xFDBAC09654320FECDEEC6CD9n,
  'The result of (-0xFEDCBA97n * -0xFEDCBA987654320Fn) is 0xFDBAC09654320FECDEEC6CD9n'
);

assert.sameValue(
  -0xFEDCBA97n * -0xFEDCBA9876543210n,
  0xFDBAC09654320FEDDDC92770n,
  'The result of (-0xFEDCBA97n * -0xFEDCBA9876543210n) is 0xFDBAC09654320FEDDDC92770n'
);

assert.sameValue(
  -0xFEDCBA98n * -0xFEDCBA98n,
  0xFDBAC096DD413A40n,
  'The result of (-0xFEDCBA98n * -0xFEDCBA98n) is 0xFDBAC096DD413A40n'
);

assert.sameValue(
  -0xFEDCBA98n * -0xFEDCBA987654320Fn,
  0xFDBAC097530ECA8555409EE8n,
  'The result of (-0xFEDCBA98n * -0xFEDCBA987654320Fn) is 0xFDBAC097530ECA8555409EE8n'
);

assert.sameValue(
  -0xFEDCBA98n * -0xFEDCBA9876543210n,
  0xFDBAC097530ECA86541D5980n,
  'The result of (-0xFEDCBA98n * -0xFEDCBA9876543210n) is 0xFDBAC097530ECA86541D5980n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn * -0xFEDCBA987654320Fn,
  0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n,
  'The result of (-0xFEDCBA987654320Fn * -0xFEDCBA987654320Fn) is 0xFDBAC097C8DC5ACAE132F7A6B7A1DCE1n'
);

assert.sameValue(
  -0xFEDCBA987654320Fn * -0xFEDCBA9876543210n,
  0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n,
  'The result of (-0xFEDCBA987654320Fn * -0xFEDCBA9876543210n) is 0xFDBAC097C8DC5ACBE00FB23F2DF60EF0n'
);

assert.sameValue(
  -0xFEDCBA9876543210n * -0xFEDCBA9876543210n,
  0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n,
  'The result of (-0xFEDCBA9876543210n * -0xFEDCBA9876543210n) is 0xFDBAC097C8DC5ACCDEEC6CD7A44A4100n'
);
