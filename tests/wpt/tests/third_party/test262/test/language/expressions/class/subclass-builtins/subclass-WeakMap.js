// This file was procedurally generated from the following sources:
// - src/subclass-builtins/WeakMap.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubWeakMap() instanceof WeakMap (Subclass instanceof Heritage)
features: [WeakMap]
flags: [generated]
---*/


const Subclass = class extends WeakMap {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof WeakMap);
