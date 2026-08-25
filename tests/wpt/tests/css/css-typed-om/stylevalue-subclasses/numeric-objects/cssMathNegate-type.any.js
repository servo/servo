// META: global=window,worker
// META: script=../../resources/testhelper.js
// META: title=CSSMathNegate.type
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#type-of-a-cssmathvalue

'use strict';

test(() => {
  const result = new CSSMathNegate(new CSSUnitValue(0, 'number'));
  assert_numeric_type_equals(result.type(), { });
}, 'Negating a type with empty map returns the empty map');

test(() => {
  const result = new CSSMathNegate(new CSSUnitValue(0, 'px'));
  assert_numeric_type_equals(result.type(), { length: 1 });
}, 'Negating a type returns the same type');
