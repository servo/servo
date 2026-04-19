// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Boolean.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubBoolean() instanceof Boolean (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends Boolean {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Boolean);
