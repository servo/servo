// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Float64Array.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubFloat64Array() instanceof Float64Array (Subclass instanceof Heritage)
features: [TypedArray, Float64Array]
flags: [generated]
---*/


const Subclass = class extends Float64Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Float64Array);
