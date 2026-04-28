// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.1.1-6-s
description: >
    Strict Mode - Use Strict Directive Prologue is ''use strict';'
    which appears in the middle of the block
flags: [noStrict]
---*/

function testcase() {
        var interface = 2;
        "use strict";
        var public = 1;

        assert.sameValue(public, 1, 'public');
        assert.sameValue(interface, 2, 'interface');
    }
testcase();
