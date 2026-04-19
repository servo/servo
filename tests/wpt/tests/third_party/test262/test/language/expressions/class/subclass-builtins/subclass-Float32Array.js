// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Float32Array.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubFloat32Array() instanceof Float32Array (Subclass instanceof Heritage)
features: [TypedArray, Float32Array]
flags: [generated]
---*/


const Subclass = class extends Float32Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Float32Array);
