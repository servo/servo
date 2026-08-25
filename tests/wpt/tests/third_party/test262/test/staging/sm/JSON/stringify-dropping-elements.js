// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

assert.sameValue(JSON.stringify({foo: 123}),
         '{"foo":123}');
assert.sameValue(JSON.stringify({foo: 123, bar: function () {}}),
         '{"foo":123}');
assert.sameValue(JSON.stringify({foo: 123, bar: function () {}, baz: 123}),
         '{"foo":123,"baz":123}');

assert.sameValue(JSON.stringify([123]),
         '[123]');
assert.sameValue(JSON.stringify([123, function () {}]),
         '[123,null]');
assert.sameValue(JSON.stringify([123, function () {}, 456]),
         '[123,null,456]');
