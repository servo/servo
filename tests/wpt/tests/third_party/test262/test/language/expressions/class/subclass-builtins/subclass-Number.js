// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Number.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubNumber() instanceof Number (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends Number {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Number);
