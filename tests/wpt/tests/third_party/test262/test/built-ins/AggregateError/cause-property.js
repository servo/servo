// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-aggregate-error
description: >
  AggregateError constructor creates own cause property
info: |
  AggregateError ( errors, message[ , options ] )

  ...
  4. Perform ? InstallErrorCause(O, options).
  ...

  InstallErrorCause ( O, options )

  1. If Type(options) is Object and ? HasProperty(options, "cause") is true, then
    a. Let cause be ? Get(options, "cause").
    b. Perform ! CreateNonEnumerableDataPropertyOrThrow(O, "cause", cause).
  ...

features: [AggregateError, error-cause]
includes: [propertyHelper.js]
---*/

var errors = [];
var message = "my-message";
var cause = { message: "my-cause" };
var error = new AggregateError(errors, message, { cause });

verifyProperty(error, "cause", {
  configurable: true,
  enumerable: false,
  writable: true,
  value: cause,
});

verifyProperty(new AggregateError(errors, message), "cause", undefined);
verifyProperty(new AggregateError(errors, message, { cause: undefined }), "cause", { value: undefined });
