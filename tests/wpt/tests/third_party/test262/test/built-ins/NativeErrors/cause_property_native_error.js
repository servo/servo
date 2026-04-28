// Copyright (C) 2021 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: NativeError constructor creates own cause property
info: |
  NativeError ( message[ , options ] )

  ...
  4. Perform ? InstallErrorCause(O, options).
  ...

  20.5.8.1 InstallErrorCause ( O, options )

  1. If Type(options) is Object and ? HasProperty(options, "cause") is true, then
    a. Let cause be ? Get(options, "cause").
    b. Perform ! CreateNonEnumerableDataPropertyOrThrow(O, "cause", cause).
  ...

esid: sec-nativeerror
features: [error-cause]
includes: [propertyHelper.js]
---*/

var nativeErrors = [
  EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError
];

for (var i = 0; i < nativeErrors.length; ++i) {
  var nativeError = nativeErrors[i];

  var message = "my-message";
  var cause = { message: "my-cause" };
  var error = new nativeError(message, { cause });

  verifyProperty(error, "cause", {
    configurable: true,
    enumerable: false,
    writable: true,
    value: cause,
  });

  verifyProperty(new nativeError(message), "cause", undefined);
  verifyProperty(new nativeError(message, { cause: undefined }), "cause", { value: undefined });
  verifyProperty(new nativeError(message, {}), "cause", undefined);
}
