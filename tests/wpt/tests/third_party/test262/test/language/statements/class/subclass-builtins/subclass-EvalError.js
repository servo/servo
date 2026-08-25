// This file was procedurally generated from the following sources:
// - src/subclass-builtins/EvalError.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubEvalError() instanceof EvalError (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends EvalError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof EvalError);
