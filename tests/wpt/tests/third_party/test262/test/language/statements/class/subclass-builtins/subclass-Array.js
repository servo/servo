// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Array.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubArray() instanceof Array (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends Array {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Array);
