// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Non-string values of `Symbol.toStringTag` property are ignored
es6id: 19.1.3.6
info: |
    16. Let tag be Get (O, @@toStringTag).
    17. ReturnIfAbrupt(tag).
    18. If Type(tag) is not String, let tag be builtinTag.
    19. Return the String that is the result of concatenating "[object ", tag,
        and "]".
features: [Symbol.toStringTag]
---*/

var custom = {};

custom[Symbol.toStringTag] = undefined;
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = null;
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = Symbol.toStringTag;
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = 86;
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = new String('test262');
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = {};
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');

custom[Symbol.toStringTag] = {
  toString: function() {
    return 'str';
  }
};
assert.sameValue(Object.prototype.toString.call(custom), '[object Object]');
