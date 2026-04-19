// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Error constructor creates own cause property
info: |
  Error ( message [ , options ] )

  ...
  4. Perform ? InstallErrorCause(O, options).
  ...

  20.5.8.1 InstallErrorCause ( O, options )

  1. If Type(options) is Object and ? HasProperty(options, "cause") is true, then
    a. Let cause be ? Get(options, "cause").
    b. Perform ! CreateNonEnumerableDataPropertyOrThrow(O, "cause", cause).
  ...

esid: sec-error-message
features: [error-cause]
includes: [propertyHelper.js]
---*/

var message = "my-message";
var cause = { message: "my-cause" };
var error = new Error(message, { cause });

verifyProperty(error, "cause", {
  configurable: true,
  enumerable: false,
  writable: true,
  value: cause,
});

verifyProperty(new Error(message), "cause", undefined);
verifyProperty(new Error(message, { cause: undefined }), "cause", { value: undefined });
