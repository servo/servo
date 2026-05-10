// This file was procedurally generated from the following sources:
// - src/subclass-builtins/TypeError.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubTypeError() instanceof TypeError (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends TypeError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof TypeError);
