'use strict';

test(() => {
  assert_true(document.modelContext instanceof ModelContext);
}, 'document.modelContext instanceof ModelContext');

test(() => {
  assert_equals(document.modelContext, document.modelContext);
}, 'document.modelContext SameObject');
