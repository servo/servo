'use strict';

test(() => {
  function empty() {
    return 'empty';
  }

  assert_throws_dom(
    'InvalidStateError',
    () => {
      navigator.modelContext.registerTool({
        name: 'empty',
        description: 'echo empty',
        execute: empty,
      });

      navigator.modelContext.registerTool({
        name: 'empty',
        description: 'echo empty',
        execute: empty,
      });
    },
    'duplicate tool registration is invalid.',
  );
}, 'duplicate tool registration is invalid.');
