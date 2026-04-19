// This file was procedurally generated from the following sources:
// - src/subclass-builtins/WeakSet.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubWeakSet() instanceof WeakSet (Subclass instanceof Heritage)
features: [WeakSet]
flags: [generated]
---*/


const Subclass = class extends WeakSet {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof WeakSet);
