// Wait for a declarative WebMCP tool to register with the specified name.
async function waitForTool(name) {
  let tools = await navigator.modelContext.getTools();
  if (tools.some(t => t.name === name)) {
    return;
  }
  await new Promise(resolve => {
    const handler = async () => {
      let tools = await navigator.modelContext.getTools();
      if (tools.some(t => t.name === name)) {
        navigator.modelContext.removeEventListener('toolchange', handler);
        resolve();
      }
    };
    navigator.modelContext.addEventListener('toolchange', handler);
  });
}

// Wait for the active WebMCP tool's input schema to match the expected schema string.
async function waitForFormToolSchemaToMatch(expected_schema) {
  await new Promise(resolve => {
    const ac = new AbortController();
    navigator.modelContext.addEventListener('toolchange', async e => {
      const [tool] = await navigator.modelContext.getTools();
      if (tool && tool.inputSchema === expected_schema) {
        resolve();
        ac.abort();
      }
    }, {signal: ac.signal});
  });
}
