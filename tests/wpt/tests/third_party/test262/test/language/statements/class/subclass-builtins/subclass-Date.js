// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Date.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubDate(0) instanceof Date (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Date {}

const sub = new Subclass(0);
assert(sub instanceof Subclass);
assert(sub instanceof Date);
