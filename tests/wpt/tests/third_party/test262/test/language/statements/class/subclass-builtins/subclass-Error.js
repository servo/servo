// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Error.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubError() instanceof Error (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Error {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Error);
