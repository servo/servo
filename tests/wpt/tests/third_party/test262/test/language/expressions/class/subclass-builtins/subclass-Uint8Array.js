// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Uint8Array.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubUint8Array() instanceof Uint8Array (Subclass instanceof Heritage)
features: [TypedArray, Uint8Array]
flags: [generated]
---*/


const Subclass = class extends Uint8Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Uint8Array);
