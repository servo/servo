// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-error.prototype.tostring
description: >
    Error.prototype.toString should throw when name or message getter throws
info: |
    Error.prototype.toString ( )

    1. Let _O_ be the *this* value.

    3. Let _name_ be ? Get(_O_, *"name"*).

    5. Let _msg_ be ? Get(_O_, *"message"*).
---*/
//Check #1
assert.throws(Test262Error, function() {
    Error.prototype.toString.call({get name () {throw new Test262Error;}});
}, "name field getter should throw to cover abrupt completion return case in step 3")

//Check #2
assert.throws(Test262Error, function() {
    Error.prototype.toString.call({get message () {throw new Test262Error;}});
}, "message field getter should throw to cover abrupt completion return case in step 5")
