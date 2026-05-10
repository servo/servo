// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Classes are always strict mode. Check computed property names and heritage
// expressions as well.

class a { constructor() { Object.preventExtensions({}).prop = 0; } }
assert.throws(TypeError, () => new a());
var aExpr = class { constructor() { Object.preventExtensions().prop = 0; } };
assert.throws(TypeError, () => new aExpr());

function shouldThrowCPN() {
    class b {
        [Object.preventExtensions({}).prop = 4]() { }
        constructor() { }
    }
}
function shouldThrowCPNExpr() {
    var b = class {
        [Object.preventExtensions({}).prop = 4]() { }
        constructor() { }
    };
}
assert.throws(TypeError, shouldThrowCPN);
assert.throws(TypeError, shouldThrowCPNExpr);

function shouldThrowHeritage() {
    class b extends (Object.preventExtensions({}).prop = 4) {
        constructor() { }
    }
}
function shouldThrowHeritageExpr() {
    var b = class extends (Object.preventExtensions({}).prop = 4) {
        constructor() { }
    };
}
assert.throws(TypeError, shouldThrowHeritage);
assert.throws(TypeError, shouldThrowHeritageExpr);

