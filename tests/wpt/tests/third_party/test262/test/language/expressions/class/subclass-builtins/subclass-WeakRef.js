// This file was procedurally generated from the following sources:
// - src/subclass-builtins/WeakRef.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubWeakRef() instanceof WeakRef (Subclass instanceof Heritage)
features: [WeakRef]
flags: [generated]
---*/


const Subclass = class extends WeakRef {}

const sub = new Subclass({});
assert(sub instanceof Subclass);
assert(sub instanceof WeakRef);
