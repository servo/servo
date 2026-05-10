// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: Observable operations from string coercion
info: |
    1. Let string be ? ToString(string).
---*/

var log, obj;

log = '';
obj = {
  toString: function() {
    log += 'toString';
  },
  valueOf: function() {
    log += 'valueOf';
  }
};

unescape(obj);

assert.sameValue(log, 'toString');

log = '';
obj = {
  toString: null,
  valueOf: function() {
    log += 'valueOf';
  }
};

unescape(obj);

assert.sameValue(log, 'valueOf');

log = '';
obj = {
  toString: function() {
    log += 'toString';
    return {};
  },
  valueOf: function() {
    log += 'valueOf';
  }
};

unescape(obj);

assert.sameValue(log, 'toStringvalueOf');
