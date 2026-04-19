// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.tostring
description: >
    Error.prototype.toString throws TypeError when 'message' field cannot be converted to a primitive
info: |
    Error.prototype.toString ( )

    1. Let _O_ be the *this* value.
    
    5. Let _msg_ be ? Get(_O_, *"message"*).

    6. If _msg_ is *undefined*, set _msg_ to the empty String; otherwise set _msg_ to ? ToString(_msg_).
    
    ToString (_argument_)

    10. Let _primValue_ be ? ToPrimitive(_argument_, ~string~).
---*/
assert.throws(TypeError, function() {
    Error.prototype.toString.call({message: {valueOf: undefined, toString: undefined}});
}, "ToPrimitive(msg) called by ToString(msg) throws TypeError")
