// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Int8Array.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubInt8Array() instanceof Int8Array (Subclass instanceof Heritage)
features: [TypedArray, Int8Array]
flags: [generated]
---*/


class Subclass extends Int8Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Int8Array);
