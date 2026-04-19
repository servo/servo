// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class C {
  get prop_this() { return this; }
}

var g_prop_this = 'prop_this';
class D extends C {
  super_prop() { return super.prop_this; }
  super_elem() { return super[g_prop_this]; }
}

var barsym = Symbol("bar");

// Test that primitive |this| values are not boxed, and undefined/null are not
// globals for super.property.
assert.sameValue(new D().super_prop.call(3), 3);
assert.sameValue(new D().super_prop.call("foo"), "foo");
assert.sameValue(new D().super_prop.call(true), true);
assert.sameValue(new D().super_prop.call(barsym), barsym);
assert.sameValue(new D().super_prop.call(null), null);
assert.sameValue(new D().super_prop.call(undefined), undefined);

// Ditto for super[elem]
assert.sameValue(new D().super_elem.call(3), 3);
assert.sameValue(new D().super_elem.call("foo"), "foo");
assert.sameValue(new D().super_elem.call(true), true);
assert.sameValue(new D().super_elem.call(barsym), barsym);
assert.sameValue(new D().super_elem.call(null), null);
assert.sameValue(new D().super_elem.call(undefined), undefined);

