// This file was procedurally generated from the following sources:
// - src/subclass-builtins/RegExp.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubRegExp() instanceof RegExp (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends RegExp {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof RegExp);
