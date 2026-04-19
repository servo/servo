// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Function.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubFunction() instanceof Function (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends Function {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Function);
