// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Boolean.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubBoolean() instanceof Boolean (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Boolean {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Boolean);
