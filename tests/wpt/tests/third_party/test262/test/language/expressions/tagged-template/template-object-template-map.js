// Copyright (C) 2018 Andrea Giammarchi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-gettemplateobject
description: >
  Template objects are canonicalized separately for each realm using its Realm Record's [[TemplateMap]]. Each [[Site]] value is a Parse Node that is a TemplateLiteral
info: |
  Let rawStrings be TemplateStrings of templateLiteral with argument true.
  Let realm be the current Realm Record.
  Let templateRegistry be realm.[[TemplateMap]].
    For each element e of templateRegistry, do
      If e.[[Site]] is the same Parse Node as templateLiteral, then
        Return e.[[Array]].

---*/
var expect;
var cache = [];
var site = 1;
function sameSite() {
  tag`${site++}`;
}

function tag(parameter) {
  if (!expect) {
    expect = parameter;
  }
  cache.push(parameter);
}

sameSite();
sameSite();
tag`${1}`;
sameSite();
sameSite();

assert(cache[0] === expect);
assert(cache[1] === expect);
assert(cache[2] !== expect);
assert(cache[3] === expect);
assert(cache[4] === expect);
