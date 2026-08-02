// META: global=window,worker
// META: title=CSSUnparsedValue: Don't crash for empty values
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#dom-cssunparsedvalue-cssunparsedvalue

'use strict';

// https://crbug.com/1169941
test(() => {
  const result = new CSSUnparsedValue(['']);
  assert_equals('', result.toString()); // Don't crash.
}, `Don't crash when serializing empty CSSUnparsedValue`);
