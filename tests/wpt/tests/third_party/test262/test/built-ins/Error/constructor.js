// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Error constructor creates own properties in sequence
info: |
  Error ( message [ , options ] )

  ...
  4. Perform ? InstallErrorCause(O, options).
  ...

esid: sec-error-message
features: [error-cause]
includes: [compareArray.js]
---*/

var message = "my-message";
var cause = { message: "my-cause" };

var sequence = [];
new Error(
  {
    toString() {
      sequence.push("toString");
      return message;
    },
  },
  {
    get cause() {
      sequence.push("cause");
      return cause;
    },
  },
);

assert.compareArray(sequence, [ "toString", "cause" ], "accessing own properties on sequence");
