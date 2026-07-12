// Wait for a declarative WebMCP tool to register with the specified name.
async function waitForTool(name) {
  const tools = await document.modelContext.getTools();
  const tool = tools.find(t => t.name === name);
  if (tool) {
    return tool;
  }
  return new Promise(resolve => {
    const handler = async () => {
      const tools = await document.modelContext.getTools();
      const tool = tools.find(t => t.name === name);
      if (tool) {
        document.modelContext.removeEventListener('toolchange', handler);
        resolve(tool);
      }
    };
    document.modelContext.addEventListener('toolchange', handler);
  });
}

// Wait for the active WebMCP tool's input schema to match the expected schema string.
async function waitForFormToolSchemaToMatch(expected_schema) {
  await new Promise(resolve => {
    const ac = new AbortController();
    document.modelContext.addEventListener('toolchange', async e => {
      const [tool] = await document.modelContext.getTools();
      if (tool && tool.inputSchema === expected_schema) {
        resolve();
        ac.abort();
      }
    }, {signal: ac.signal});
  });
}
