'use strict';

function test_register_tool_schema_error(inputSchema, expectedError, testName) {
  test(() => {
    assert_throws_js(
      expectedError,
      () => {
        navigator.modelContext.registerTool({
          name: 'empty',
          description: 'empty',
          inputSchema,
          execute: () => {},
        });
      },
      `Should throw ${expectedError.name} for invalid schema`,
    );
  }, testName);
}

test_register_tool_schema_error(
  { toJSON: () => undefined },
  TypeError,
  'registerTool throws when inputSchema.toJSON() returns undefined',
);

test_register_tool_schema_error(
  (() => {
    const circular = {};
    circular.self = circular;
    return circular;
  })(),
  TypeError,
  'registerTool throws when inputSchema contains a circular reference',
);

test_register_tool_schema_error(
  BigInt(42),
  TypeError,
  'registerTool throws when inputSchema contains non-serializable types (BigInt)',
);

test(() => {
  // Register with invalid schema AND aborted signal. This asserts that input
  // schema validation happens before checking the aborted signal.
  const circularSchema = {};
  circularSchema.self = circularSchema;

  assert_throws_js(
    TypeError,
    () => {
      navigator.modelContext.registerTool(
        {
          name: 'aborted_invalid_schema',
          description: 'empty',
          inputSchema: {
            type: "object",
            properties: {
              prop1: circularSchema,
            }
          },
          execute: () => {},
        },
        { signal: AbortSignal.abort('aborted') }
      );
    },
    'Should throw TypeError for circular input schema, before checking if the signal is aborted'
  );
}, 'registerTool throws on invalid schema even if an aborted signal was provided');
