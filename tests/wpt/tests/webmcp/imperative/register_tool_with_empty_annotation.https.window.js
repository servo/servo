'use strict';

test(() => {
  const tool = {
    name: 'echo',
    description: 'echo input',
    execute: (obj) => obj.text,
    annotations: {
      // No `readOnlyHint` member.
    },
  };

  const controller = new AbortController();
  navigator.modelContext.registerTool(tool, { signal: controller.signal });
  controller.abort();
}, 'register tool with empty annotations');
