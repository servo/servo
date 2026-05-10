// META: title=Language Model Tool Use (Open Loop)
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

// `Tool Use` Tests - Open-loop tool-calling pattern.
// The model proposes tool calls as part of the response, and the
// application is responsible for executing tools and providing results
// via follow-up prompts.

// Constant for triggering tool call generation in the echo model.
const TOOL_CALL_TRIGGER = '<GenerateSimpleToolCalls>';
// Constant for triggering multiple tool call batches in the echo model.
const MULTIPLE_TOOL_CALL_TRIGGER = '<GenerateMultipleToolCalls>';

promise_test(async t => {
  await ensureLanguageModel();

  // Test with null inputSchema.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with null inputSchema.",
      inputSchema: null
    }]
  }));
}, 'createLanguageModel should reject when tool has null inputSchema.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with empty object inputSchema - missing required 'type' property.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [{
      name: "testTool",
      description: "Test tool with empty object inputSchema. Args: {}",
      inputSchema: {}
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema is empty object without type property.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with non-object inputSchema.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with string inputSchema.",
      inputSchema: "not an object"
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema is not an object.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with inputSchema missing 'type' property.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with inputSchema missing type.",
      inputSchema: {
        properties: {
          param: { type: "string" }
        }
      }
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema has no type property.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with inputSchema type not 'object'.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with non-object inputSchema type.",
      inputSchema: {
        type: "string"
      }
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema type is not object.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with invalid properties structure.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with invalid properties structure.",
      inputSchema: {
        type: "object",
        properties: "not an object"
      }
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema properties is not an object.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with invalid required array.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool with invalid required structure.",
      inputSchema: {
        type: "object",
        properties: {
          param: { type: "string" }
        },
        required: "not an array"
      }
    }]
  }));
}, 'createLanguageModel should reject when tool inputSchema required is not an array.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with inputSchema that throws on property access (Proxy trap).
  const throwingSchema = new Proxy({type: "object"}, {
    get(target, prop) {
      if (prop === "properties") {
        throw new Error("Proxy trap threw on property access");
      }
      return target[prop];
    }
  });

  await promise_rejects_js(t, Error, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [{
      name: "proxyTool",
      description: "Tool with throwing Proxy inputSchema.",
      inputSchema: throwingSchema
    }]
  }));
}, 'createLanguageModel should propagate exception when inputSchema getter throws.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with inputSchema that throws on "type" property access.
  const throwingTypeSchema = new Proxy({}, {
    get(target, prop) {
      if (prop === "type") {
        throw new TypeError("Cannot read type property");
      }
      return target[prop];
    }
  });

  await promise_rejects_js(t, TypeError, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [{
      name: "proxyTypeTool",
      description: "Tool with throwing Proxy on type access.",
      inputSchema: throwingTypeSchema
    }]
  }));
}, 'createLanguageModel should propagate exception when inputSchema type getter throws.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with empty tool name.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "",
      description: "Test tool with empty name.",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  }));
}, 'createLanguageModel should reject when tool has empty name.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with empty description.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  }));
}, 'createLanguageModel should reject when tool has empty description.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with duplicate tool names - should reject.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [
      {
        name: "duplicateTool",
        description: "First tool with this name.",
        inputSchema: {
          type: "object",
          properties: {}
        }
      },
      {
        name: "duplicateTool", // Same name as above - should cause rejection
        description: "Second tool with duplicate name.",
        inputSchema: {
          type: "object",
          properties: {}
        }
      }
    ]
  }));
}, 'createLanguageModel should reject when tools array contains duplicate tool names.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with missing inputSchema.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool without inputSchema."
    }]
  }));
}, 'createLanguageModel should reject when tool has no inputSchema.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with circular reference in inputSchema.
  // JSON.stringify will fail on circular references, and this should now
  // reject with a TypeError exposing the error to JavaScript.
  const circularSchema = {
    type: "object",
    properties: {
      param: { type: "string" }
    }
  };
  // Create circular reference.
  circularSchema.circular = circularSchema;

  // Should reject because the circular reference cannot be serialized.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [
      {
        name: "validTool",
        description: "A valid tool. Args: {\"action\": \"test\"}",
        inputSchema: {
          type: "object",
          properties: {
            action: { type: "string" }
          }
        }
      },
      {
        name: "circularTool",
        description: "Tool with circular reference. Args: {}",
        inputSchema: circularSchema
      }
    ]
  }));
}, 'createLanguageModel should reject when tool has circular reference in inputSchema');

promise_test(async t => {
  await ensureLanguageModel();

  // Test when ALL tools have circular references (all fail JSON.stringify).
  // This should reject with TypeError because the circular references
  // cannot be serialized.
  const circularSchema1 = {
    type: "object",
    properties: { param1: { type: "string" } }
  };
  circularSchema1.circular = circularSchema1;

  const circularSchema2 = {
    type: "object",
    properties: { param2: { type: "string" } }
  };
  circularSchema2.circular = circularSchema2;

  // All tools have circular references - should reject with TypeError.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [
      {
        name: "circularTool1",
        description: "First tool with circular reference. Args: {}",
        inputSchema: circularSchema1
      },
      {
        name: "circularTool2",
        description: "Second tool with circular reference. Args: {}",
        inputSchema: circularSchema2
      }
    ]
  }));
}, 'createLanguageModel should reject when all tools have circular references');

promise_test(async t => {
  await ensureLanguageModel();

  // Test when a custom toJSON() method throws an exception.
  // The V8 exception should propagate to JavaScript instead of being
  // replaced with a generic error message.
  const schemaWithThrowingToJSON = {
    type: "object",
    properties: {
      param: { type: "string" }
    },
    toJSON() {
      throw new Error("Custom toJSON error - this should propagate");
    }
  };

  await promise_rejects_js(t, Error, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [
      {
        name: "throwingTool",
        description: "Tool with throwing toJSON. Args: {}",
        inputSchema: schemaWithThrowingToJSON
      }
    ]
  }), "Custom toJSON error should propagate");
}, 'createLanguageModel should propagate V8 exception from custom toJSON()');

promise_test(async t => {
  await ensureLanguageModel();

  // Test when a custom getter throws an exception during JSON.stringify.
  // The V8 exception should propagate to JavaScript.
  const schemaWithThrowingGetter = {
    type: "object",
    properties: {
      param: { type: "string" }
    },
    get throwingProperty() {
      throw new Error("Getter threw during serialization");
    }
  };

  await promise_rejects_js(t, Error, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [
      {
        name: "throwingGetterTool",
        description: "Tool with throwing getter. Args: {}",
        inputSchema: schemaWithThrowingGetter
      }
    ]
  }), "Getter exception should propagate");
}, 'createLanguageModel should propagate V8 exception from custom getter during serialization');

promise_test(async t => {
  await ensureLanguageModel();

  // Test fail-fast behavior: when one tool (among multiple) has an error,
  // the entire create() operation should fail rather than silently skipping
  // the invalid tool. This ensures all-or-nothing semantics.
  const circularSchema = {
    type: "object",
    properties: { param: { type: "string" } }
  };
  circularSchema.circular = circularSchema;

  await promise_rejects_js(t, TypeError, createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [
      {
        name: "validTool1",
        description: "First valid tool. Args: {\"action\": \"test\"}",
        inputSchema: {
          type: "object",
          properties: {
            action: { type: "string" }
          }
        }
      },
      {
        name: "invalidTool",
        description: "Tool with circular reference. Args: {}",
        inputSchema: circularSchema
      },
      {
        name: "validTool2",
        description: "Second valid tool. Args: {\"count\": 42}",
        inputSchema: {
          type: "object",
          properties: {
            count: { type: "number" }
          }
        }
      }
    ]
  }));
}, 'createLanguageModel should fail-fast when one tool among many is invalid');

promise_test(async t => {
  await ensureLanguageModel();

  // V8's JSON.stringify properly escapes special characters,
  // demonstrating that the invalid JSON examples cannot be produced:
  //   JSON.parse('{"a": 10,}')
  //   JSON.parse('{"a": 10,\n  // Comment.\n  "b": 11\n}')
  // Even if the schema description contains these exact strings as text
  // content, JSON.stringify escapes them properly, producing valid RFC JSON.
  const schemaWithInvalidJSONLikeContent = {
    type: "object",
    properties: {
      trailingCommaExample: {
        type: "string",
        description: '{"a": 10,}'  // Contains trailing comma as string content.
      },
      commentExample: {
        type: "string",
        description: '{\n  "a": 10,\n  // Comment.\n  "b": 11\n}'  // Contains comment as string content.
      }
    }
  };

  // Should succeed - JSON.stringify escapes quotes, newlines, etc. producing
  // valid JSON.
  const model = await createLanguageModel({
    expectedOutputs: [{ type: 'tool-call' }],
    tools: [{
      name: "exampleTool",
      description: "Demonstrates JSON.stringify escaping of some invalid JSON examples. Args: {}",
      inputSchema: schemaWithInvalidJSONLikeContent
    }]
  });
  assert_true(!!model, 'Model should be created successfully with invalid-JSON-like string content');
}, 'createLanguageModel with schema containing invalid-JSON-like text (trailing commas, comments) as string content succeeds because JSON.stringify escapes properly');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with explicitly empty tools array - should succeed.
  const model = await createLanguageModel({
    tools: []
  });

  // Should be able to use the model normally.
  const result = await model.prompt('Hello');
  assert_equals(typeof result, 'string', 'Result should be a string');
  assert_true(result.includes('Hello'), 'Result should echo back the input "Hello"');
}, 'createLanguageModel should succeed with empty tools array.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with no tools property - should succeed.
  const model = await createLanguageModel({});

  // Should be able to use the model normally.
  const result = await model.prompt('Hello');
  assert_equals(typeof result, 'string', 'Result should be a string');
  assert_true(result.includes('Hello'), 'Result should echo back the input "Hello"');
}, 'createLanguageModel should succeed with no tools property.');

// expectedOutputs validation tests with tools.

promise_test(async t => {
  await ensureLanguageModel();

  // Test with tools but no expectedOutputs - should reject.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool.",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
    // Missing expectedOutputs
  }));
}, 'createLanguageModel should reject when tools provided but expectedOutputs is missing.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with tools but expectedOutputs doesn't include tool-call - should
  // reject.
  await promise_rejects_js(t, TypeError, createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool.",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }],
    expectedOutputs: [
      { type: "text" }  // Missing tool-call type.
    ]
  }));
}, 'createLanguageModel should reject when tools provided but expectedOutputs does not include tool-call.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with tools and expectedOutputs includes tool-call - should succeed.
  const model = await createLanguageModel({
    tools: [{
      name: "testTool",
      description: "Test tool.",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }],
    expectedOutputs: [
      { type: "tool-call" }
    ]
  });

  assert_true(!!model, 'Model should be created successfully');
}, 'createLanguageModel should succeed when tools provided with tool-call in expectedOutputs.');

promise_test(async t => {
  await ensureLanguageModel();

  // Test with expectedOutputs includes tool-call but no tools - should succeed.
  // (Will return text since no tools are available)
  const model = await createLanguageModel({
    expectedOutputs: [
      { type: "tool-call" }
    ]
    // No tools
  });

  assert_true(!!model, 'Model should be created successfully');

  // Should work normally. Since no tools are available, will return text string.
  const result = await model.prompt('Hello');
  assert_equals(typeof result, 'string', 'Result should be a string when no tools available');
  assert_true(result.includes('Hello'), 'Result should echo back the input "Hello"');
}, 'createLanguageModel should succeed with tool-call in expectedOutputs but no tools.');

// Open-loop tool-calling pattern tests.
// The EchoAILanguageModel generates tool calls based on argument hints
// provided in tool descriptions using the format: "Args: {JSON}"

promise_test(async t => {
  await ensureLanguageModel();

  // Test basic tool call with open-loop pattern.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }  // Required when tools are provided.
    ],
    tools: [{
      name: "get_weather",
      description: "Get the weather in a location. Args: {\"location\": \"Seattle\"}",
      inputSchema: {
        type: "object",
        properties: {
          location: {
            type: "string",
            description: "The city to check for the weather condition."
          }
        },
        required: ["location"]
      }
    }]
  });

  // Trigger tool call using the explicit trigger prefix.
  const result = await model.prompt(TOOL_CALL_TRIGGER + 'What is the weather in Seattle?');

  // Returns structured message with tool-call type -
  // sequence<LanguageModelMessageContent>.
  assert_true(Array.isArray(result), 'Result should be an array of messages');
  assert_true(result.length > 0, 'Result should have at least one message');

  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_equals(typeof toolCall.callID, 'string', 'Tool call should have callID');
  assert_equals(toolCall.name, 'get_weather', 'Tool call name should be get_weather');
  assert_equals(typeof toolCall.arguments, 'object', 'Tool call arguments should be an object');
  assert_equals(toolCall.arguments.location, 'Seattle', 'Tool call should have location=Seattle');
}, 'prompt() should return structured tool call messages in open-loop pattern');

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "calculator",
      description: "Evaluate a mathematical expression. Args: {\"expression\": \"2 + 2\"}",
      inputSchema: {
        type: "object",
        properties: {
          expression: {
            type: "string",
            description: "A mathematical expression."
          }
        },
        required: ["expression"]
      }
    }]
  });

  // First call - get tool call using the explicit trigger prefix.
  const firstResult = await model.prompt(TOOL_CALL_TRIGGER + 'What is 2 + 2?');
  assert_true(Array.isArray(firstResult), 'First result should be an array');

  const toolCallMessage = firstResult.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_equals(toolCall.name, 'calculator', 'Tool call name should match');
  assert_equals(toolCall.arguments.expression, '2 + 2',
    'Tool call arguments should match the echoed hint');
  const callID = toolCall.callID;

  // Execute tool (simulated).
  const toolResult = "4";

  // Send tool response back via open-loop pattern.
  const secondResult = await model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolSuccess({
          callID: callID,
          name: 'calculator',
          result: [{ type: 'text', value: toolResult }]
        })
      }]
    }
  ]);

  // Model should process the tool response.
  assert_equals(typeof secondResult, 'string', 'Second result should be a string');
  assert_true(secondResult.includes('4'), 'Response should include the result "4"');
}, 'Open-loop pattern - send tool response via follow-up prompt');

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "test-tool",
      description: "Test tool. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  // Create an actual ImageBitmap (DOM object with internal fields).
  const canvas = document.createElement('canvas');
  canvas.width = 4;
  canvas.height = 4;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = 'red';
  ctx.fillRect(0, 0, 4, 4);
  const imageBitmap = await createImageBitmap(canvas);

  // Invalid: Passing DOM object (ImageBitmap) with type='object' instead of
  // type='image'.
  const toolSuccess = new LanguageModelToolSuccess({
    callID: 'test-123',
    name: 'test-tool',
    result: [
      { type: 'object', value: imageBitmap }  // Wrong type for ImageBitmap!
    ]
  });

  await promise_rejects_dom(t, 'DataError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: toolSuccess
      }]
    }
  ]), 'DOM object with incorrect type should throw DataError');
}, 'Tool response with DOM object (ImageBitmap) labeled as type object should reject');

// TODO(crbug.com/422803232): Adjust expectations when image is supported in
// LanguageModelToolSuccess.
promise_test(async t => {
  await ensureLanguageModel();

  // Test multimodal tool response with image content.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "generate_image",
      description: "Generate a simple image. Args: {\"color\": \"red\"}",
      inputSchema: {
        type: "object",
        properties: {
          color: {
            type: "string",
            description: "Color of the image."
          }
        },
        required: ["color"]
      }
    }]
  });

  // First call - get tool call.
  const firstResult = await model.prompt(TOOL_CALL_TRIGGER + 'Generate a red image');
  assert_true(Array.isArray(firstResult), 'First result should be an array');

  const toolCallMessage = firstResult.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_equals(toolCall.name, 'generate_image', 'Tool call name should match');
  const callID = toolCall.callID;

  // Simulate tool execution - create an ImageBitmap.
  const canvas = document.createElement('canvas');
  canvas.width = 100;
  canvas.height = 100;
  const ctx = canvas.getContext('2d');
  ctx.fillStyle = 'red';
  ctx.fillRect(0, 0, 100, 100);
  const imageBitmap = await createImageBitmap(canvas);

  // Send multimodal tool response with image.
  const toolResponse = new LanguageModelToolSuccess({
    callID: callID,
    name: 'generate_image',
    result: [
      { type: 'text', value: 'Generated a red image:' },
      { type: 'image', value: imageBitmap }
    ]
  });

  await promise_rejects_dom(t, 'NotSupportedError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: toolResponse
      }]
    }
  ]), 'Image type in tool response should throw NotSupportedError');
}, 'Multimodal tool response with ImageBitmap throws NotSupportedError');

// TODO(crbug.com/422803232): Adjust expectations when audio is supported in
// LanguageModelToolSuccess.
promise_test(async t => {
  await ensureLanguageModel();

  // Test multimodal tool response with audio content.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "generate_audio",
      description: "Generate a simple audio tone. Args: {\"frequency\": \"440\"}",
      inputSchema: {
        type: "object",
        properties: {
          frequency: {
            type: "string",
            description: "Frequency of the tone in Hz."
          }
        },
        required: ["frequency"]
      }
    }]
  });

  // First call - get tool call.
  const firstResult = await model.prompt(TOOL_CALL_TRIGGER + 'Generate a 440Hz tone');
  assert_true(Array.isArray(firstResult), 'First result should be an array');

  const toolCallMessage = firstResult.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_equals(toolCall.name, 'generate_audio', 'Tool call name should match');
  const callID = toolCall.callID;

  // Verify the tool call arguments contain the frequency.
  assert_equals(typeof toolCall.arguments, 'object', 'Tool call arguments should be an object');
  assert_equals(toolCall.arguments.frequency, '440', 'Tool call should have frequency=440');

  // Simulate tool execution - create an AudioBuffer using the frequency from arguments.
  const frequency = parseInt(toolCall.arguments.frequency, 10);
  const audioContext = new AudioContext();
  const sampleRate = audioContext.sampleRate;
  const duration = 1; // 1 second
  const audioBuffer = audioContext.createBuffer(
    1,  // channels
    sampleRate * duration,
    sampleRate
  );

  // Fill with a sine wave at the requested frequency.
  const channelData = audioBuffer.getChannelData(0);
  for (let i = 0; i < audioBuffer.length; i++) {
    channelData[i] = Math.sin(2 * Math.PI * frequency * i / sampleRate);
  }

  // Send multimodal tool response with audio.
  const toolResponse = new LanguageModelToolSuccess({
    callID: callID,
    name: 'generate_audio',
    result: [
      { type: 'text', value: 'Generated a 440Hz tone:' },
      { type: 'audio', value: audioBuffer }
    ]
  });

  await promise_rejects_dom(t, 'NotSupportedError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: toolResponse
      }]
    }
  ]), 'Audio type in tool response should throw NotSupportedError');
}, 'Multimodal tool response with AudioBuffer throws NotSupportedError');

promise_test(async t => {
  await ensureLanguageModel();

  // Test streaming with tool calls.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "get_time",
      description: "Get current time. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  // Use promptStreaming.
  const stream = model.promptStreaming(TOOL_CALL_TRIGGER + 'What time is it?');
  const reader = stream.getReader();

  let messages = [];
  let done = false;

  while (!done) {
    const { value, done: readerDone } = await reader.read();
    done = readerDone;
    if (value) {
      messages.push(value);
    }
  }

  // Should have received tool call message chunks.
  assert_true(messages.length > 0, 'Should have received messages');

  const toolCallChunks = messages.filter(msg => msg.type === 'tool-call');
  assert_true(toolCallChunks.length > 0, 'Should have received at least one tool-call chunk');

  const firstToolCall = toolCallChunks[0];
  assert_equals(typeof firstToolCall.value.callID, 'string', 'Tool call should have callID');
  assert_equals(firstToolCall.value.name, 'get_time', 'Tool call name should be get_time');
}, 'promptStreaming() should stream tool call messages');

promise_test(async t => {
  await ensureLanguageModel();

  // Test tools that take no arguments.
  // Test both valid formats:
  // {type: "object", properties: {}} and {type: "object"}.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [
      {
        name: "noArgumentsTool",
        description: "A tool that takes no arguments. Args: {}",
        inputSchema: {
          type: "object",
          properties: {}
        }
      },
      {
        name: "noArgumentsToolMinimal",
        description: "A tool that takes no arguments (minimal format). Args: {}",
        inputSchema: {
          type: "object"
        }
      }
    ]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Use the noArgumentsTool');
  assert_true(Array.isArray(result), 'Result should be an array');

  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_true(toolCall.name === 'noArgumentsTool' || toolCall.name === 'noArgumentsToolMinimal',
              'Tool call name should match one of the parameter-less tools');
  assert_equals(typeof toolCall.arguments, 'object', 'Arguments should be an object');
  assert_equals(Object.keys(toolCall.arguments).length, 0, 'Arguments should be empty');
}, 'Tool with no arguments should have empty arguments object');

promise_test(async t => {
  await ensureLanguageModel();

  // Test multiple tools.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [
      {
        name: "get_weather",
        description: "Get the weather in a location. Args: {\"location\": \"Seattle\"}",
        inputSchema: {
          type: "object",
          properties: {
            location: { type: "string" }
          },
          required: ["location"]
        }
      },
      {
        name: "get_traffic",
        description: "Get traffic information. Args: {\"location\": \"Seattle\"}",
        inputSchema: {
          type: "object",
          properties: {
            location: { type: "string" }
          },
          required: ["location"]
        }
      }
    ]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Check weather and traffic for Seattle');
  assert_true(Array.isArray(result), 'Result should be an array');

  const toolCallMessages = result.filter(msg => msg.type === 'tool-call');
  assert_true(toolCallMessages.length >= 1, 'Should have at least one tool call');

  // Verify tool names are from the declared tools.
  const toolNames = toolCallMessages.map(msg => msg.value.name);
  toolNames.forEach(name => {
    assert_true(['get_weather', 'get_traffic'].includes(name),
                `Tool name ${name} should be one of the declared tools`);
  });
}, 'Multiple tools can be declared and called');

promise_test(async t => {
  await ensureLanguageModel();

  // Test error handling in tool response.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "errorTool",
      description: "A tool that may error. Args: {\"action\": \"fail\"}",
      inputSchema: {
        type: "object",
        properties: {
          action: { type: "string" }
        },
        required: ["action"]
      }
    }]
  });

  const firstResult = await model.prompt(TOOL_CALL_TRIGGER + 'Use errorTool');
  const toolCallMessage = firstResult.find(msg => msg.type === 'tool-call');
  const callID = toolCallMessage.value.callID;

  // Send error response.
  const secondResult = await model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolError({
          callID: callID,
          name: 'errorTool',
          errorMessage: 'Tool execution failed'
        })
      }]
    }
  ]);

  // Model should handle the error response.
  assert_equals(typeof secondResult, 'string', 'Should return a string response');
}, 'Tool response can include error field');

// Tool response serialization error handling tests.

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "testTool",
      description: "Test tool. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Use testTool');
  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  const callID = toolCallMessage.value.callID;

  // Create a circular reference in tool result value.
  const circularObj = {};
  circularObj.self = circularObj;

  // Sending tool response with circular reference should reject with DataError.
  await promise_rejects_dom(t, 'DataError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolSuccess({
          callID: callID,
          name: 'testTool',
          result: [{ type: 'text', value: circularObj }]
        })
      }]
    }
  ]));
}, 'Tool response with circular reference should reject with DataError');

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "testTool",
      description: "Test tool. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Use testTool');
  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  const callID = toolCallMessage.value.callID;

  // Sending tool response with function should reject with DataError.
  await promise_rejects_dom(t, 'DataError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolSuccess({
          callID: callID,
          name: 'testTool',
          result: [{ type: 'text', value: function() {} }]
        })
      }]
    }
  ]));
}, 'Tool response with function value should reject with DataError');

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "testTool",
      description: "Test tool. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Use testTool');
  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  const callID = toolCallMessage.value.callID;

  // Sending tool response with BigInt should reject with DataError.
  await promise_rejects_dom(t, 'DataError', model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolSuccess({
          callID: callID,
          name: 'testTool',
          result: [{ type: 'text', value: 12345678901234567890n }]
        })
      }]
    }
  ]));
}, 'Tool response with BigInt value should reject with DataError');

promise_test(async t => {
  await ensureLanguageModel();

  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "testTool",
      description: "Test tool. Args: {}",
      inputSchema: {
        type: "object",
        properties: {}
      }
    }]
  });

  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Use testTool');
  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  const callID = toolCallMessage.value.callID;

  // Sending valid tool response should succeed.
  const secondResult = await model.prompt([
    {
      role: 'user',
      content: [{
        type: 'tool-response',
        value: new LanguageModelToolSuccess({
          callID: callID,
          name: 'testTool',
          result: [
            { type: 'text', value: 'Valid string result' },
            { type: 'text', value: { nested: 'object', data: 123 } },
            { type: 'text', value: [1, 2, 3, 'array'] }
          ]
        })
      }]
    }
  ]);

  assert_equals(typeof secondResult, 'string', 'Valid tool response should succeed');
}, 'Tool response with valid serializable values should succeed');

promise_test(async t => {
  await ensureLanguageModel();

  // Test cloning model with tools.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "testTool",
      description: "Test tool. Args: {\"param\": \"value\"}",
      inputSchema: {
        type: "object",
        properties: {
          param: { type: "string" }
        }
      }
    }]
  });

  const clonedModel = await model.clone();
  assert_true(clonedModel instanceof LanguageModel, 'Cloned model should be a LanguageModel');

  // Cloned model should also support tool calls.
  const result = await clonedModel.prompt(TOOL_CALL_TRIGGER + 'Use testTool');
  assert_true(Array.isArray(result), 'Cloned model should return array with tool calls');

  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Cloned model should generate tool calls');
}, 'Cloned model should preserve tools');

promise_test(async t => {
  await ensureLanguageModel();

  // Test model returning both text and tool call in the same response.
  // The echo model echoes the input text (with trigger stripped) before
  // emitting tool calls, demonstrating the simplified open-loop pattern.
  const model = await createLanguageModel({
    expectedInputs: [
      { type: 'tool-response' }
    ],
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [{
      name: "search",
      description: "Search for information. Args: {\"query\": \"test\"}",
      inputSchema: {
        type: "object",
        properties: {
          query: {
            type: "string",
            description: "The search query."
          }
        },
        required: ["query"]
      }
    }]
  });

  // Trigger a response that includes both text and tool call.
  const result = await model.prompt(TOOL_CALL_TRIGGER + 'Can you search for chromium?');

  // Should return an array when tool calls are present.
  assert_true(Array.isArray(result), 'Result should be an array of messages when tool call is present');
  assert_true(result.length >= 2, 'Result should have at least 2 messages (text + tool-call)');

  // First message should be the echoed input text (trigger stripped).
  const textMessage = result.find(msg => msg.type === 'text');
  assert_true(!!textMessage, 'Should have a text message');
  assert_equals(typeof textMessage.value, 'string', 'Text message value should be a string');
  assert_true(textMessage.value.includes('Can you search for chromium'),
    'Text should include the echoed input (with trigger stripped)');

  // Should also have a tool-call message.
  const toolCallMessage = result.find(msg => msg.type === 'tool-call');
  assert_true(!!toolCallMessage, 'Should have a tool-call message');

  const toolCall = toolCallMessage.value;
  assert_equals(typeof toolCall.callID, 'string', 'Tool call should have callID');
  assert_equals(toolCall.name, 'search', 'Tool call name should be search');
  assert_equals(typeof toolCall.arguments, 'object', 'Tool call arguments should be an object');
  assert_equals(toolCall.arguments.query, 'test', 'Tool call should use argument hint from description');

  // Verify order: text message should come before tool-call message.
  const textIndex = result.findIndex(msg => msg.type === 'text');
  const toolCallIndex = result.findIndex(msg => msg.type === 'tool-call');
  assert_true(textIndex < toolCallIndex, 'Text message should come before tool-call message');
}, 'prompt() returns both text and tool call in correct order when model outputs mixed response');

promise_test(async t => {
  await ensureLanguageModel();

  // Create model with multiple tools to test batch splitting.
  const model = await createLanguageModel({
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [
      {
        name: "tool1",
        description: "First tool. Args: {\"arg1\": \"value1\"}",
        inputSchema: {
          type: "object",
          properties: {
            arg1: { type: "string" }
          }
        }
      },
      {
        name: "tool2",
        description: "Second tool. Args: {\"arg2\": \"value2\"}",
        inputSchema: {
          type: "object",
          properties: {
            arg2: { type: "string" }
          }
        }
      },
      {
        name: "tool3",
        description: "Third tool. Args: {\"arg3\": \"value3\"}",
        inputSchema: {
          type: "object",
          properties: {
            arg3: { type: "string" }
          }
        }
      },
      {
        name: "tool4",
        description: "Fourth tool. Args: {\"arg4\": \"value4\"}",
        inputSchema: {
          type: "object",
          properties: {
            arg4: { type: "string" }
          }
        }
      }
    ]
  });

  // Use the multiple tool call trigger.
  const result = await model.prompt(MULTIPLE_TOOL_CALL_TRIGGER + 'Test multiple batches');

  assert_true(Array.isArray(result), 'Result should be an array');
  assert_true(result.length > 0, 'Should have received messages');

  // Count tool call messages.
  const toolCallMessages = result.filter(msg => msg.type === 'tool-call');
  assert_equals(toolCallMessages.length, 4, 'Should have received all 4 tool calls from both batches');

  // Verify all tool names are present.
  const toolNames = toolCallMessages.map(msg => msg.value.name);
  assert_true(toolNames.includes('tool1'), 'Should have tool1');
  assert_true(toolNames.includes('tool2'), 'Should have tool2');
  assert_true(toolNames.includes('tool3'), 'Should have tool3');
  assert_true(toolNames.includes('tool4'), 'Should have tool4');

  // Verify all have valid callIDs.
  for (const msg of toolCallMessages) {
    assert_equals(typeof msg.value.callID, 'string', 'Tool call should have callID');
    assert_true(msg.value.callID.length > 0, 'Tool call should have non-empty callID');
  }
}, 'prompt() should handle multiple batches of tool calls from model');

promise_test(async t => {
  await ensureLanguageModel();

  // Create model with multiple tools.
  const model = await createLanguageModel({
    expectedOutputs: [
      { type: 'tool-call' }
    ],
    tools: [
      {
        name: "streamTool1",
        description: "First streaming tool. Args: {}",
        inputSchema: {
          type: "object",
          properties: {}
        }
      },
      {
        name: "streamTool2",
        description: "Second streaming tool. Args: {}",
        inputSchema: {
          type: "object",
          properties: {}
        }
      },
      {
        name: "streamTool3",
        description: "Third streaming tool. Args: {}",
        inputSchema: {
          type: "object",
          properties: {}
        }
      },
      {
        name: "streamTool4",
        description: "Fourth streaming tool. Args: {}",
        inputSchema: {
          type: "object",
          properties: {}
        }
      }
    ]
  });

  // Use the multiple tool call trigger with streaming.
  const stream = model.promptStreaming(MULTIPLE_TOOL_CALL_TRIGGER + 'Test streaming batches');
  const reader = stream.getReader();

  let messages = [];
  let done = false;

  while (!done) {
    const { value, done: readerDone } = await reader.read();
    done = readerDone;
    if (value) {
      messages.push(value);
    }
  }

  assert_true(messages.length > 0, 'Should have received messages');

  // Count tool call chunks.
  const toolCallChunks = messages.filter(msg => msg.type === 'tool-call');
  assert_equals(toolCallChunks.length, 4, 'Should have received all 4 tool calls from both batches');

  // Verify all tool names are present.
  const toolNames = toolCallChunks.map(msg => msg.value.name);
  assert_true(toolNames.includes('streamTool1'), 'Should have streamTool1');
  assert_true(toolNames.includes('streamTool2'), 'Should have streamTool2');
  assert_true(toolNames.includes('streamTool3'), 'Should have streamTool3');
  assert_true(toolNames.includes('streamTool4'), 'Should have streamTool4');

  // Verify all have valid callIDs.
  for (const msg of toolCallChunks) {
    assert_equals(typeof msg.value.callID, 'string', 'Tool call should have callID');
    assert_true(msg.value.callID.length > 0, 'Tool call should have non-empty callID');
  }
}, 'promptStreaming() should handle multiple batches of tool calls from model');
