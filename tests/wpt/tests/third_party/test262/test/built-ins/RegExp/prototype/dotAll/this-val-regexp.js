// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.dotall
description: >
    `dotAll` accessor function invoked on a RegExp instance
info: |
    21.2.5.12 get RegExp.prototype.dotAll

    4. Let flags be the value of R’s [[OriginalFlags]] internal slot.
    5. If flags contains the code unit "s", return true.
    6. Return false.
features: [regexp-dotall]
---*/

assert.sameValue(/./.dotAll, false, "/./.dotAll");
assert.sameValue(/./i.dotAll, false, "/./i.dotAll");
assert.sameValue(/./g.dotAll, false, "/./g.dotAll");
assert.sameValue(/./y.dotAll, false, "/./y.dotAll");
assert.sameValue(/./m.dotAll, false, "/./m.dotAll");

assert.sameValue(/./s.dotAll, true, "/./s.dotAll");
assert.sameValue(/./is.dotAll, true, "/./is.dotAll");
assert.sameValue(/./sg.dotAll, true, "/./sg.dotAll");
assert.sameValue(/./sy.dotAll, true, "/./sy.dotAll");
assert.sameValue(/./ms.dotAll, true, "/./ms.dotAll");

assert.sameValue(new RegExp(".", "").dotAll, false, "new RegExp('.', '').dotAll");
assert.sameValue(new RegExp(".", "i").dotAll, false, "new RegExp('.', 'i').dotAll");
assert.sameValue(new RegExp(".", "g").dotAll, false, "new RegExp('.', 'g').dotAll");
assert.sameValue(new RegExp(".", "y").dotAll, false, "new RegExp('.', 'y').dotAll");
assert.sameValue(new RegExp(".", "m").dotAll, false, "new RegExp('.', 'm').dotAll");

assert.sameValue(new RegExp(".", "s").dotAll, true, "new RegExp('.', 's').dotAll");
assert.sameValue(new RegExp(".", "is").dotAll, true, "new RegExp('.', 'is').dotAll");
assert.sameValue(new RegExp(".", "sg").dotAll, true, "new RegExp('.', 'sg').dotAll");
assert.sameValue(new RegExp(".", "sy").dotAll, true, "new RegExp('.', 'sy').dotAll");
assert.sameValue(new RegExp(".", "ms").dotAll, true, "new RegExp('.', 'ms').dotAll");
