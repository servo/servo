// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.2.5.12
description: >
    `sticky` property descriptor
info: |
    RegExp.prototype.sticky is an accessor property whose set accessor
    function is undefined.
---*/

var desc = Object.getOwnPropertyDescriptor(RegExp.prototype, 'sticky');

assert.sameValue(desc.set, undefined);
assert.sameValue(typeof desc.get, 'function');
