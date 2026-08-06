// META: global=window,worker
// META: script=../../resources/testhelper.js
// META: title=CSSMathInvert.type
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#type-of-a-cssmathvalue

'use strict';

test(() => {
  const result = new CSSMathInvert(new CSSUnitValue(0, 'number'));
  assert_numeric_type_equals(result.type(), { });
}, 'Inverting a type with empty map returns the empty map');

test(() => {
  const x = new CSSMathProduct(new CSSUnitValue(0, 'px'), new CSSUnitValue(0, 's'));
  const result = new CSSMathInvert(x);
  assert_numeric_type_equals(result.type(), { length: -1, time: -1 });
}, 'Inverting a type negates all its exponents');

test(() => {
  const x = new CSSUnitValue(0, 'px');
  const result = new CSSMathInvert(new CSSMathInvert(x));
  assert_numeric_type_equals(result.type(), { length: 1 });
}, 'Inverting an inverted type returns the original type');
