// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: >
  Throws a TypeError if Intl.DisplayNames is called as a function.
info: |
  Intl.DisplayNames ([ locales [ , options ]])

  1. If NewTarget is undefined, throw a TypeError exception.
  ...
features: [Intl.DisplayNames]
---*/

assert.throws(TypeError, function() {
  Intl.DisplayNames();
});

assert.throws(TypeError, function() {
  Intl.DisplayNames('en');
});

assert.throws(TypeError, function() {
  Intl.DisplayNames(['en']);
});
