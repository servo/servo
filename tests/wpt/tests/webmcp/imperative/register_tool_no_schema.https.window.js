'use strict';

test(() => {
  const tool = {
    name: 'empty',
    description: 'echo empty',
    execute: () => {},
  };

  const controller = new AbortController();
  document.modelContext.registerTool(tool, { signal: controller.signal });
  controller.abort();
}, 'register tool with only required params');
