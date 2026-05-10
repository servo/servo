// This file was procedurally generated from the following sources:
// - src/subclass-builtins/URIError.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubURIError() instanceof URIError (Subclass instanceof Heritage)
flags: [generated]
---*/


class Subclass extends URIError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof URIError);
