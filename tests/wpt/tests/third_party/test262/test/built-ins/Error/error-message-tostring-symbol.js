// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error-message
description: >
    Error throws TypeError when _message_ is a Symbol.
info: |
    Error ( _message_ [ , _options_ ] )

    3. If _message_ is not *undefined*, then
        a. Let _msg_ be ? ToString(_message_).

    ToString (_argument_)

    2. If _argument_ is a Symbol, throw a *TypeError* exception.
features: [Symbol]
---*/

assert.throws(TypeError, function () {
    Error(Symbol())
}, "If _message_ is a Symbol, Error must throw TypeError");
