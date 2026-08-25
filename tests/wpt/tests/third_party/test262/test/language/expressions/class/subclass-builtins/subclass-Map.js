// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Map.case
// - src/subclass-builtins/default/expression.template
/*---
description: new SubMap() instanceof Map (Subclass instanceof Heritage)
features: [Map]
flags: [generated]
---*/


const Subclass = class extends Map {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Map);
