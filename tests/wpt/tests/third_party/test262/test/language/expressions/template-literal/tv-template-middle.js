// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 11.8.6.1
description: Template values of the template middle pattern
info: |
    The TV and TRV of TemplateMiddle :: }${ is the empty code unit sequence.
    The TRV of TemplateMiddle :: } TemplateCharacters ${ is the TRV of
    TemplateCharacters.
---*/

var calls;

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[1], '', 'Template value (empty)');
  assert.sameValue(s.raw[1], '', 'Template raw value (empty)');
})`${1}${2}`;
assert.sameValue(calls, 1);

calls = 0;
(function(s) {
  calls++;
  assert.sameValue(s[1], 'foo', 'Template value (with content)');
  assert.sameValue(s.raw[1], 'foo', 'Template raw value (with content)');
})`${1}foo${2}`;
assert.sameValue(calls, 1);
