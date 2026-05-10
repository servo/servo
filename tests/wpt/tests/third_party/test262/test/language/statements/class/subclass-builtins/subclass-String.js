// This file was procedurally generated from the following sources:
// - src/subclass-builtins/String.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubString() instanceof String (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends String {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof String);
