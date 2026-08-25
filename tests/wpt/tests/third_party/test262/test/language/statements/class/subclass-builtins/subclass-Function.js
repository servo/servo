// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Function.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubFunction() instanceof Function (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Function {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Function);
