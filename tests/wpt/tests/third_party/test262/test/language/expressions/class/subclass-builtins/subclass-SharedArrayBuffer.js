// This file was procedurally generated from the following sources:
// - src/subclass-builtins/SharedArrayBuffer.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubSharedArrayBuffer() instanceof SharedArrayBuffer (Subclass instanceof Heritage)
features: [SharedArrayBuffer]
flags: [generated]
---*/


const Subclass = class extends SharedArrayBuffer {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof SharedArrayBuffer);
