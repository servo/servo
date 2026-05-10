// Copyright (C) 2019 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors when calling @@matchAll
info: |
  String.prototype.matchAll ( regexp )
    [...]
    2. If _regexp_ is neither *undefined* nor *null*, then
             1. Let _isRegExp_ be ? IsRegExp(_regexp_).
             1. If _isRegExp_ is true, then
               1. Let _flags_ be ? Get(_regexp_, *"flags"*).
               1. Perform ? RequireObjectCoercible(_flags_).
               1. If ? ToString(_flags_) does not contain *"g"*, throw a *TypeError* exception.
features: [Symbol.matchAll]
---*/

var regex = /a/g;
Object.defineProperty(regex, 'flags', { value: undefined });

assert.throws(TypeError, function () {
  ''.matchAll(regex);
});

Object.defineProperty(RegExp.prototype, 'flags', {
  get: function () {
    return undefined;
  }
});

assert.throws(TypeError, function () {
  ''.matchAll(/a/g);
});
