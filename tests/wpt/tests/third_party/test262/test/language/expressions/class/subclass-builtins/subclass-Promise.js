// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Promise.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubPromise() instanceof Promise (Subclass instanceof Heritage)
features: [Promise]
flags: [generated]
---*/


const Subclass = class extends Promise {}

const sub = new Subclass(() => {});
assert(sub instanceof Subclass);
assert(sub instanceof Promise);
