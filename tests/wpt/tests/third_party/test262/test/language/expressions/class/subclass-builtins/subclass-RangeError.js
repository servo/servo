// This file was procedurally generated from the following sources:
// - src/subclass-builtins/RangeError.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubRangeError() instanceof RangeError (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends RangeError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof RangeError);
