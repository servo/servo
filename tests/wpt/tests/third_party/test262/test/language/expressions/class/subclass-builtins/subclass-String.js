// This file was procedurally generated from the following sources:
// - src/subclass-builtins/String.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubString() instanceof String (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends String {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof String);
