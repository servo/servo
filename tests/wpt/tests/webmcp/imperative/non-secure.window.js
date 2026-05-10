'use strict';

test(() => {
  assert_false(navigator.hasOwnProperty('modelContext'));
}, `navigator.modelContext is not supported in non-secure context.`);
