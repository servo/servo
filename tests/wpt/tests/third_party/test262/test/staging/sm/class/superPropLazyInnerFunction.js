// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
testcase();
function testcase() {
    var tokenCodes  = {
      get try() {
        try {
          super.actual();
        } catch (e) {}
      }
    };
    var arr = [
        'try',
    ];
    for (var i = 0; i < arr.length; i++) {
        if (tokenCodes[arr[i]] !== i) {};
    }
}

