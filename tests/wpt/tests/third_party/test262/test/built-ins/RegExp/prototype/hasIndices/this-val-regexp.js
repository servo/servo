// Copyright (C) 2021 Ron Buckton and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.hasindices
description: >
    `hasIndices` accessor function invoked on a RegExp instance
info: |
    21.2.5.12 get RegExp.prototype.hasIndices

    4. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    5. If flags contains the code unit "s", return true.
    6. Return false.
features: [regexp-match-indices]
---*/

assert.sameValue(/./.hasIndices, false, "/./.hasIndices");
assert.sameValue(/./i.hasIndices, false, "/./i.hasIndices");
assert.sameValue(/./g.hasIndices, false, "/./g.hasIndices");
assert.sameValue(/./y.hasIndices, false, "/./y.hasIndices");
assert.sameValue(/./m.hasIndices, false, "/./m.hasIndices");
assert.sameValue(/./s.hasIndices, false, "/./s.hasIndices");
assert.sameValue(/./u.hasIndices, false, "/./u.hasIndices");

assert.sameValue(/./d.hasIndices, true, "/./d.hasIndices");
assert.sameValue(/./di.hasIndices, true, "/./di.hasIndices");
assert.sameValue(/./dg.hasIndices, true, "/./dg.hasIndices");
assert.sameValue(/./dy.hasIndices, true, "/./dy.hasIndices");
assert.sameValue(/./dm.hasIndices, true, "/./dm.hasIndices");
assert.sameValue(/./ds.hasIndices, true, "/./ds.hasIndices");
assert.sameValue(/./du.hasIndices, true, "/./du.hasIndices");

assert.sameValue(new RegExp(".", "").hasIndices, false, "new RegExp('.', '').hasIndices");
assert.sameValue(new RegExp(".", "i").hasIndices, false, "new RegExp('.', 'i').hasIndices");
assert.sameValue(new RegExp(".", "g").hasIndices, false, "new RegExp('.', 'g').hasIndices");
assert.sameValue(new RegExp(".", "y").hasIndices, false, "new RegExp('.', 'y').hasIndices");
assert.sameValue(new RegExp(".", "m").hasIndices, false, "new RegExp('.', 'm').hasIndices");
assert.sameValue(new RegExp(".", "s").hasIndices, false, "new RegExp('.', 's').hasIndices");
assert.sameValue(new RegExp(".", "u").hasIndices, false, "new RegExp('.', 'u').hasIndices");

assert.sameValue(new RegExp(".", "d").hasIndices, true, "new RegExp('.', 'd').hasIndices");
assert.sameValue(new RegExp(".", "di").hasIndices, true, "new RegExp('.', 'di').hasIndices");
assert.sameValue(new RegExp(".", "dg").hasIndices, true, "new RegExp('.', 'dg').hasIndices");
assert.sameValue(new RegExp(".", "dy").hasIndices, true, "new RegExp('.', 'dy').hasIndices");
assert.sameValue(new RegExp(".", "dm").hasIndices, true, "new RegExp('.', 'dm').hasIndices");
assert.sameValue(new RegExp(".", "ds").hasIndices, true, "new RegExp('.', 'ds').hasIndices");
assert.sameValue(new RegExp(".", "du").hasIndices, true, "new RegExp('.', 'du').hasIndices");
