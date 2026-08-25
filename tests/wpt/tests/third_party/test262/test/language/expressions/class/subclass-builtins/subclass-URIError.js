// This file was procedurally generated from the following sources:
// - src/subclass-builtins/URIError.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubURIError() instanceof URIError (Subclass instanceof Heritage)
flags: [generated]
---*/


const Subclass = class extends URIError {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof URIError);
