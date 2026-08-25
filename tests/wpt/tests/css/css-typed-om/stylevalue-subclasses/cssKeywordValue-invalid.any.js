// META: global=window,worker
// META: title=CSSKeywordValue Error Handling
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#csskeywordvalue

'use strict';

test(() => {
  assert_throws_js(TypeError, () => new CSSKeywordValue(''));
}, 'Constructing CSSKeywordValue with an empty string throws a TypeError');
