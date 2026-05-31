'use strict';

test(() => {
  assert_false(navigator.hasOwnProperty('modelContext'));
}, `document.modelContext is not supported in non-secure context.`);
