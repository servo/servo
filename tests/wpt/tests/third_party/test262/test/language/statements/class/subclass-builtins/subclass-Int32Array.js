// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Int32Array.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubInt32Array() instanceof Int32Array (Subclass instanceof Heritage)
features: [TypedArray, Int32Array]
flags: [generated]
---*/


class Subclass extends Int32Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Int32Array);
