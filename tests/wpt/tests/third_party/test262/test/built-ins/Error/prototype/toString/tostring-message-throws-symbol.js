// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.tostring
description: >
    Error.prototype.toString throws a TypeError when message field is a Symbol.
info: |
    Error.prototype.toString ( )

    1. Let _O_ be the *this* value.
    
    5. Let _msg_ be ? Get(_O_, *"message"*).

    6. If _msg_ is *undefined*, set _msg_ to the empty String; otherwise set _msg_ to ? ToString(_msg_).
    
    ToString (_argument_)

    2. If _argument_ is a Symbol, throw a *TypeError* exception.
features: [Symbol]
---*/
assert.throws(TypeError, function () {
    Error.prototype.toString.call({message : Symbol()})
}, "If message field is a symbol, Error.prototype.toString must throw a TypeError")
