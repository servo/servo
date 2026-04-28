// This file was procedurally generated from the following sources:
// - src/subclass-builtins/TypeError.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubTypeError() instanceof TypeError (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends TypeError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof TypeError);
