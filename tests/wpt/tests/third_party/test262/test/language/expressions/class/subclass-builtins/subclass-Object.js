// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Object.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubObject() instanceof Object (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends Object {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Object);
