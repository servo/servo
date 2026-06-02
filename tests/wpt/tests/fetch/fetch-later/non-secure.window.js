'use strict';

test(() => {
  assert_false(window.hasOwnProperty('fetchLater'));
}, `fetchLater() is not supported in non-secure context.`);
