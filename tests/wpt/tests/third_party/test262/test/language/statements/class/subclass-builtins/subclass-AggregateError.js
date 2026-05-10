// This file was procedurally generated from the following sources:
// - src/subclass-builtins/AggregateError.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubAggregateError() instanceof AggregateError (Subclass instanceof Heritage)
features: [AggregateError]
flags: [generated]
---*/


class Subclass extends AggregateError {}

const sub = new Subclass([]);
assert(sub instanceof Subclass);
assert(sub instanceof AggregateError);
