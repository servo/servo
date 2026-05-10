// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    String values of `Symbol.toStringTag` property override built-in tags
es6id: 19.1.3.6
info: |
    1. If the this value is undefined, return "[object Undefined]".
    2. If the this value is null, return "[object Null]".

    14. Else, let builtinTag be "Object".
    15. Let tag be ? Get(O, @@toStringTag).
    16. If Type(tag) is not String, let tag be builtinTag.
    17. Return the String that is the result of concatenating "[object ", tag, and "]".

    4.3.2 primitive value

    member of one of the types Undefined, Null, Boolean, Number, Symbol, or String as defined in clause 6

features: [Symbol.toStringTag]
---*/


Boolean.prototype[Symbol.toStringTag] = 'test262';
assert.sameValue(Object.prototype.toString.call(Boolean.prototype), '[object test262]');
assert.sameValue(Object.prototype.toString.call(true), '[object test262]');

Number.prototype[Symbol.toStringTag] = 'test262';
assert.sameValue(Object.prototype.toString.call(Number.prototype), '[object test262]');
assert.sameValue(Object.prototype.toString.call(0), '[object test262]');

String.prototype[Symbol.toStringTag] = 'test262';
assert.sameValue(Object.prototype.toString.call(String.prototype), '[object test262]');
assert.sameValue(Object.prototype.toString.call(''), '[object test262]');


Object.defineProperty(Symbol.prototype, Symbol.toStringTag, {
  value: 'test262'
});

assert.sameValue(Object.prototype.toString.call(Symbol.prototype), '[object test262]');
