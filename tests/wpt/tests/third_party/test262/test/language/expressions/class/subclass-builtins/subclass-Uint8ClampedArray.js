// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Uint8ClampedArray.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubUint8ClampedArray() instanceof Uint8ClampedArray (Subclass instanceof Heritage)
features: [TypedArray, Uint8ClampedArray]
flags: [generated]
---*/


const Subclass = class extends Uint8ClampedArray {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Uint8ClampedArray);
