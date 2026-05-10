'use strict';

test(() => {
  assert_true(navigator.modelContext instanceof ModelContext);
}, 'navigator.modelContext instanceof ModelContext');

test(() => {
  assert_equals(navigator.modelContext, navigator.modelContext);
}, 'navigator.modelContext SameObject');
