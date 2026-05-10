// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.prototype.finally
description: >
    SpeciesConstructor throws a TypeError if the constructor property is not an object.
info: |
    Promise.prototype.finally ( _onFinally_ )

    1. Let _promise_ be the *this* value.
    2. If _promise_ is not an Object, throw a *TypeError* exception.
    3. Let _C_ be ? SpeciesConstructor(_promise_, %Promise%).

    SpeciesConstructor (_O_,_defaultConstructor_)

    1. Let _C_ be ? Get(_O_, *"constructor"*).
    2. If _C_ is *undefined*, return _defaultConstructor_.
    3. If _C_ is not an Object, throw a *TypeError* exception.
features: [Promise, Promise.prototype.finally]
---*/
assert.throws(TypeError, function () {
    Promise.prototype.finally.call({constructor : 0})
}, "SpeciesConstructor throws a TypeError if the constructor property is not an object.")
