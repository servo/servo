// Copyright (C) 2022 Mathias Bynens, Ron Buckton, and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.unicodesets
description: >
    `unicodeSets` accessor function invoked on a RegExp instance
info: |
    get RegExp.prototype.unicodeSets -> RegExpHasFlag

    4. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    5. If flags contains the code unit "s", return true.
    6. Return false.
features: [regexp-v-flag]
---*/

assert.sameValue(/./.unicodeSets, false, "/./.unicodeSets");
assert.sameValue(/./d.unicodeSets, false, "/./d.unicodeSets");
assert.sameValue(/./g.unicodeSets, false, "/./g.unicodeSets");
assert.sameValue(/./i.unicodeSets, false, "/./i.unicodeSets");
assert.sameValue(/./m.unicodeSets, false, "/./m.unicodeSets");
assert.sameValue(/./s.unicodeSets, false, "/./s.unicodeSets");
assert.sameValue(/./u.unicodeSets, false, "/./u.unicodeSets");
assert.sameValue(/./y.unicodeSets, false, "/./y.unicodeSets");

assert.sameValue(/./v.unicodeSets, true, "/./v.unicodeSets");
assert.sameValue(/./vd.unicodeSets, true, "/./vd.unicodeSets");
assert.sameValue(/./vg.unicodeSets, true, "/./vg.unicodeSets");
assert.sameValue(/./vi.unicodeSets, true, "/./vi.unicodeSets");
assert.sameValue(/./vm.unicodeSets, true, "/./vm.unicodeSets");
assert.sameValue(/./vs.unicodeSets, true, "/./vs.unicodeSets");
// Note: `/vu` throws an early parse error and is tested separately.
assert.sameValue(/./vy.unicodeSets, true, "/./vy.unicodeSets");

assert.sameValue(new RegExp(".", "").unicodeSets, false, "new RegExp('.', '').unicodeSets");
assert.sameValue(new RegExp(".", "d").unicodeSets, false, "new RegExp('.', 'd').unicodeSets");
assert.sameValue(new RegExp(".", "g").unicodeSets, false, "new RegExp('.', 'g').unicodeSets");
assert.sameValue(new RegExp(".", "i").unicodeSets, false, "new RegExp('.', 'i').unicodeSets");
assert.sameValue(new RegExp(".", "m").unicodeSets, false, "new RegExp('.', 'm').unicodeSets");
assert.sameValue(new RegExp(".", "s").unicodeSets, false, "new RegExp('.', 's').unicodeSets");
assert.sameValue(new RegExp(".", "u").unicodeSets, false, "new RegExp('.', 'u').unicodeSets");
assert.sameValue(new RegExp(".", "y").unicodeSets, false, "new RegExp('.', 'y').unicodeSets");

assert.sameValue(new RegExp(".", "v").unicodeSets, true, "new RegExp('.', 'v').unicodeSets");
assert.sameValue(new RegExp(".", "vd").unicodeSets, true, "new RegExp('.', 'vd').unicodeSets");
assert.sameValue(new RegExp(".", "vg").unicodeSets, true, "new RegExp('.', 'vg').unicodeSets");
assert.sameValue(new RegExp(".", "vi").unicodeSets, true, "new RegExp('.', 'vi').unicodeSets");
assert.sameValue(new RegExp(".", "vm").unicodeSets, true, "new RegExp('.', 'vm').unicodeSets");
assert.sameValue(new RegExp(".", "vs").unicodeSets, true, "new RegExp('.', 'vs').unicodeSets");
// Note: `new RegExp(pattern, 'vu')` throws a runtime error and is tested separately.
assert.sameValue(new RegExp(".", "vy").unicodeSets, true, "new RegExp('.', 'vy').unicodeSets");
