// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Number.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubNumber() instanceof Number (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Number {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Number);
