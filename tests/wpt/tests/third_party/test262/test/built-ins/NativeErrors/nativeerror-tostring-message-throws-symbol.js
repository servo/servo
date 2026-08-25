// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-nativeerror
description: >
    NativeError constructors throw TypeError when _message_ is a Symbol
info: |
    _NativeError_ ( _message_ [ , _options_ ] )

    3. If _message_ is not *undefined*, then
        a. Let _msg_ be ? ToString(_message_).

    ToString (_argument_)

    2. If _argument_ is a Symbol, throw a *TypeError* exception.
features: [Symbol]
---*/
//Check #1
assert.throws(TypeError, function() {
    EvalError(Symbol())
}, "If _message_ is a Symbol, EvalError should throw a TypeError")

//Check #2
assert.throws(TypeError, function() {
    RangeError(Symbol())
}, "If _message_ is a Symbol, RangeError should throw a TypeError")

//Check #3
assert.throws(TypeError, function() {
    ReferenceError(Symbol())
}, "If _message_ is a Symbol, ReferenceError should throw a TypeError")

//Check #4
assert.throws(TypeError, function() {
    SyntaxError(Symbol())
}, "If _message_ is a Symbol, SyntaxError should throw a TypeError")

//Check #5
assert.throws(TypeError, function() {
    TypeError(Symbol())
}, "If _message_ is a Symbol, TypeError should throw a TypeError")

//Check #6
assert.throws(TypeError, function() {
    URIError(Symbol())
}, "If _message_ is a Symbol, URIError should throw a TypeError")
