// This file was procedurally generated from the following sources:
// - src/subclass-builtins/Map.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubMap() instanceof Map (Subclass instanceof Heritage)
features: [Map]
flags: [generated]
---*/


class Subclass extends Map {}

const sub = new Subclass();
assert(sub instanceof Subclass);
assert(sub instanceof Map);
