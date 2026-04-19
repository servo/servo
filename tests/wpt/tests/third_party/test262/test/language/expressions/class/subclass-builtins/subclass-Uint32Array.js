// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Uint32Array.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubUint32Array() instanceof Uint32Array (Subclass instanceof Heritage)
features: [TypedArray, Uint32Array]
flags: [generated]
---*/


const Subclass = class extends Uint32Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Uint32Array);
