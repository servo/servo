// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: NativeError constructor creates own message property
info: |
  19.5.6.1.1 NativeError ( message )

  ...
  4.
    ...
    c. Let msgDesc be the PropertyDescriptor{[[Value]]: msg, [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true}.
    d. Let status be DefinePropertyOrThrow(O, "message", msgDesc).
es6id: 19.5.6.1.1
includes: [propertyHelper.js]
---*/

var nativeErrors = [
  EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError
];

for (var i = 0; i < nativeErrors.length; ++i) {
  var nativeError = nativeErrors[i];

  var message = "my-message";
  var error = new nativeError(message);

  verifyEqualTo(error, "message", message);
  verifyNotEnumerable(error, "message");
  verifyWritable(error, "message");
  verifyConfigurable(error, "message");
}
