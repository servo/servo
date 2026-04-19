// This file was procedurally generated from the following sources:
// - src/subclass-builtins/BigInt64Array.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubBigInt64Array() instanceof BigInt64Array (Subclass instanceof Heritage)
features: [TypedArray, BigInt]
flags: [generated]
---*/


class Subclass extends BigInt64Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof BigInt64Array);
