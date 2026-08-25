// This file was procedurally generated from the following sources:
// - src/subclass-builtins/SyntaxError.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubSyntaxError() instanceof SyntaxError (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends SyntaxError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof SyntaxError);
