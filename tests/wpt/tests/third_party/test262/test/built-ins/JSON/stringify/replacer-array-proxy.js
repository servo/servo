// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Array proxy replacer serves as a filter of object keys.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  4. If Type(replacer) is Object, then
     a. If IsCallable(replacer) is true, then
        i. Let ReplacerFunction be replacer.
     b. Else,
        i. Let isArray be ? IsArray(replacer).

  IsArray ( argument )

  [...]
  3. If argument is a Proxy exotic object, then
     a. If argument.[[ProxyHandler]] is null, throw a TypeError exception.
     b. Let target be argument.[[ProxyTarget]].
     c. Return ? IsArray(target).
features: [Proxy]
---*/

var replacer = new Proxy(['b'], {});

assert.sameValue(JSON.stringify({a: 1, b: 2}, replacer), '{"b":2}');
assert.sameValue(JSON.stringify({b: {a: 3, b: 4}}, replacer), '{"b":{"b":4}}');
