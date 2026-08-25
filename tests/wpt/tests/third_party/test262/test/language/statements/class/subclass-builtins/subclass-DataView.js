// This file was procedurally generated from the following sources:
// - src/subclass-builtins/DataView.case
// - src/subclass-builtins/default/statement.template
/*---
description: new SubDataView() instanceof DataView (Subclass instanceof Heritage)
features: [TypedArray, DataView]
flags: [generated]
---*/


class Subclass extends DataView {}

const sub = new Subclass(new ArrayBuffer(1));
assert(sub instanceof Subclass);
assert(sub instanceof DataView);
