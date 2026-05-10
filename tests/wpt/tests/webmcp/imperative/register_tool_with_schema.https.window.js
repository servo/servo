'use strict';

test(() => {
  const tool = {
    name: 'echo',
    description: 'echo input',
    inputSchema: {
      type: 'object',
      properties: {
        text: {
          description: 'Value to echo',
          type: 'string',
        },
      },
      required: ['text'],
    },
    execute: (obj) => obj.text,
    annotations: {
      readOnlyHint: 'true',
    },
  };

  const controller = new AbortController();
  navigator.modelContext.registerTool(tool, { signal: controller.signal });
  controller.abort();
}, 'register and unregister script tool');

test(() => {
  navigator.modelContext.registerTool({
    name: 'empty',
    description: 'empty',
    inputSchema: {
      toJSON: () => {
        return 'undefined';
      },
    },
    execute: () => {},
  });
}, "registerTool succeeds when inputSchema.toJSON() returns 'undefined'");
