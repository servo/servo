// This file was procedurally generated from the following sources:
// - src/subclass-builtins/SyntaxError.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubSyntaxError() instanceof SyntaxError (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends SyntaxError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof SyntaxError);
