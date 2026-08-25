// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class constructor property
---*/
class C {}
assert.sameValue(
    C,
    C.prototype.constructor,
    "The value of `C` is `C.prototype.constructor`"
);
var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');
assert.sameValue(desc.configurable, true, "The value of `desc.configurable` is `true`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");
assert.sameValue(desc.enumerable, false, "The value of `desc.enumerable` is `false`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");
assert.sameValue(desc.writable, true, "The value of `desc.writable` is `true`, after executing `var desc = Object.getOwnPropertyDescriptor(C.prototype, 'constructor');`");
