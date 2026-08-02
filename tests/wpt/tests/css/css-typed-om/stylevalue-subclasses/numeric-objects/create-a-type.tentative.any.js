// META: global=window,worker
// META: script=../../resources/testhelper.js
// META: title=Creating Type From A Unit
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#create-a-type

'use strict';

test(() => {
  const value = new CSSUnitValue(0, 'number');
  assert_numeric_type_equals(value.type(), {});
}, 'Creating a type from "number" returns {}');

test(() => {
  const value = new CSSUnitValue(0, 'percent');
  assert_numeric_type_equals(value.type(), { percent: 1 });
}, 'Creating a type from "percent" returns { percent: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 'px');
  assert_numeric_type_equals(value.type(), { length: 1 });
}, 'Creating a type from <length> returns { length: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 'deg');
  assert_numeric_type_equals(value.type(), { angle: 1 });
}, 'Creating a type from <angle> returns { angle: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 's');
  assert_numeric_type_equals(value.type(), { time: 1 });
}, 'Creating a type from <time> returns { time: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 'Hz');
  assert_numeric_type_equals(value.type(), { frequency: 1 });
}, 'Creating a type from <frequency> returns { frequency: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 'dpi');
  assert_numeric_type_equals(value.type(), { resolution: 1 });
}, 'Creating a type from <resolution> returns { resolution: 1 }');

test(() => {
  const value = new CSSUnitValue(0, 'fr');
  assert_numeric_type_equals(value.type(), { flex: 1 });
}, 'Creating a type from <flex> returns { flex: 1 }');
