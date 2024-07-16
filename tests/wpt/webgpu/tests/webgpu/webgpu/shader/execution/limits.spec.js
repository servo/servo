/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Execution tests for WGSL limits.`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { keysOf } from '../../../common/util/data_tables.js';
import { iterRange } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';
import { checkElementsEqualGenerated } from '../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);

// The limits that we test.
const kMaxStructMembers = 1023;
const kMaxCompositeNestingDepth = 15;
const kMaxBraceNestingDepth = 127;
const kMaxFunctionParameters = 255;
const kMaxSwitchCaseSelectors = 1023;
const kMaxPrivateStorageSize = 8192;
const kMaxFunctionStorageSize = 8192;
const kMaxConstArrayElements = 2047;

/**
 * Run a shader and check that the buffer output matches expectations.
 *
 * @param t The test object
 * @param wgsl The shader source
 * @param input The initial buffer contents
 * @param expected The expected buffer contents after running the shader
 * @param constants The optional pipeline overridable constant values
 */
function runShaderTest(
t,
wgsl,
input,
expected,
constants)
{
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main',
      constants
    }
  });

  // Allocate a buffer and copy the input values to it.
  const outputBuffer = t.makeBufferWithContents(
    input,
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  );
  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }]
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check that the output matches the expected values.
  t.expectGPUBufferValuesPassCheck(
    outputBuffer,
    (data) => checkElementsEqualGenerated(data, expected),
    {
      type: Uint32Array,
      typedLength: input.length
    }
  );
}

g.test('struct_members').
desc(`Test that structures with the maximum number of members are supported.`).
fn((t) => {
  let code = `struct S {`;
  for (let m = 0; m < kMaxStructMembers; m++) {
    code += `  m${m}: u32,\n`;
  }
  code += `}

    @group(0) @binding(0) var<storage, read_write> buffer : S;

    @compute @workgroup_size(1)
    fn main() {
      buffer = S();
    }
    `;

  runShaderTest(
    t,
    code,
    new Uint32Array([...iterRange(kMaxStructMembers, (_i) => 0xdeadbeef)]),
    (_i) => 0
  );
});

g.test('nesting_depth_composite_struct').
desc(`Test that composite types can be nested up to the maximum level.`).
fn((t) => {
  let code = `struct S0 { a : u32 }\n`;
  for (let s = 1; s < kMaxCompositeNestingDepth; s++) {
    code += `struct S${s} { a : S${s - 1} }\n`;
  }
  code += `
    @group(0) @binding(0) var<storage, read_write> buffer : S${kMaxCompositeNestingDepth - 1};

    @compute @workgroup_size(1)
    fn main() {
      buffer = S${kMaxCompositeNestingDepth - 1}();
    }
    `;

  runShaderTest(t, code, new Uint32Array([0xdeadbeef]), (_i) => 0);
});

g.test('nesting_depth_composite_array').
desc(`Test that composite types can be nested up to the maximum level.`).
fn((t) => {
  let type = ``;
  for (let m = 0; m < kMaxCompositeNestingDepth; m++) {
    type += `array<`;
  }
  type += 'u32';
  for (let m = 0; m < kMaxCompositeNestingDepth; m++) {
    type += `, 1>`;
  }

  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : ${type};

    @compute @workgroup_size(1)
    fn main() {
      buffer = ${type}();
    }
    `;

  runShaderTest(t, code, new Uint32Array([0xdeadbeef]), (_i) => 0);
});

g.test('nesting_depth_braces').
desc(`Test that brace-enclosed statements can be nested up to the maximum level.`).
fn((t) => {
  let code = `@group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${kMaxBraceNestingDepth}>;
    @compute @workgroup_size(1)

    fn main() {
    `;
  // Note: We subtract one from the spec value to account for the function body, and another one
  // for the nested statement itself.
  for (let b = 0; b < kMaxBraceNestingDepth - 2; b++) {
    code += `  {\n`;
  }
  code += `    buffer[0] = 42;\n`;
  for (let b = 0; b < kMaxBraceNestingDepth - 2; b++) {
    code += `  }\n`;
  }
  code += `
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(kMaxBraceNestingDepth, (i) => i)]), (i) => {
    return i === 0 ? 42 : i;
  });
});

g.test('function_parameters').
desc(`Test that functions can have the maximum number of parameters.`).
fn((t) => {
  let code = `@group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${kMaxFunctionParameters}>;

    fn bar(`;
  for (let p = 0; p < kMaxFunctionParameters; p++) {
    code += `p${p}: u32, `;
  }
  code += `) {`;

  for (let p = 0; p < kMaxFunctionParameters; p++) {
    code += `buffer[${p}] = p${p};\n`;
  }

  code += `}

    @compute @workgroup_size(1)
    fn main() {
      bar(`;
  for (let p = 0; p < kMaxFunctionParameters; p++) {
    code += `${p}, `;
  }
  code += `);
    }
    `;

  runShaderTest(
    t,
    code,
    new Uint32Array([...iterRange(kMaxFunctionParameters, (_i) => 0xdeadbeef)]),
    (i) => i
  );
});

g.test('switch_case_selectors').
desc(
  `Test that switch statements can have the maximum number of case selectors in separate clauses.`
).
fn((t) => {
  let code = `@group(0) @binding(0) var<storage, read_write> buffer : array<u32, 2>;

    @compute @workgroup_size(1)
    fn main() {
      switch (buffer[0]) {
        default {}`;
  for (let s = 0; s < kMaxSwitchCaseSelectors - 1; s++) {
    code += `
        case ${s} { buffer[1] = ${s}; }`;
  }
  code += `
      };
    }
    `;

  runShaderTest(t, code, new Uint32Array([42, 0xdeadbeef]), (_i) => 42);
});

g.test('switch_case_selectors_same_clause').
desc(
  `Test that switch statements can have the maximum number of case selectors in the same clause.`
).
fn((t) => {
  let code = `@group(0) @binding(0) var<storage, read_write> buffer : array<u32, 2>;

    @compute @workgroup_size(1)
    fn main() {
      switch (buffer[0]) {
        default {}
        case `;
  for (let s = 0; s < kMaxSwitchCaseSelectors - 1; s++) {
    code += `${s}, `;
  }
  code += ` { buffer[1] = 42; }
      };
    }
    `;

  runShaderTest(t, code, new Uint32Array([999, 0xdeadbeef]), (i) => {
    return i === 0 ? 999 : 42;
  });
});

// A list of types used for array elements.
const kArrayElements = {
  bool: {
    size: 4,
    to_u32: (x) => `u32(${x})`
  },
  u32: {
    size: 4,
    to_u32: (x) => x
  },
  vec4u: {
    size: 16,
    to_u32: (x) => `dot(${x}, ${x})`
  }
};

g.test('private_array_byte_size').
desc(`Test that arrays in the private address space up to the maximum size are supported.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(kMaxPrivateStorageSize / type.size);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    var<private> arr : array<${t.params.type}, ${elements}>;

    @compute @workgroup_size(1)
    fn main() {
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0);
});

g.test('private_array_combined_byte_size').
desc(`Test the combined sizes of variables in the private address space.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(kMaxPrivateStorageSize / type.size / 4);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    var<private> arr1 : array<${t.params.type}, ${elements}>;
    var<private> arr2 : array<${t.params.type}, ${elements}>;
    var<private> arr3 : array<${t.params.type}, ${elements}>;
    var<private> arr4 : array<${t.params.type}, ${elements}>;

    @compute @workgroup_size(1)
    fn main() {
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr1[i]')} + ${type.to_u32('arr2[i]')} +
                    ${type.to_u32('arr3[i]')} + ${type.to_u32('arr4[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0);
});

g.test('function_array_byte_size').
desc(`Test that arrays in the function address space up to the maximum size are supported.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(kMaxFunctionStorageSize / type.size);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    @compute @workgroup_size(1)
    fn main() {
      var arr : array<${t.params.type}, ${elements}>;
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0);
});

g.test('function_variable_combined_byte_size').
desc(`Test the combined sizes of variables in the function address space.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(kMaxFunctionStorageSize / type.size / 4);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    @compute @workgroup_size(1)
    fn main() {
      var arr1 : array<${t.params.type}, ${elements}>;
      var arr2 : array<${t.params.type}, ${elements}>;
      var arr3 : array<${t.params.type}, ${elements}>;
      var arr4 : array<${t.params.type}, ${elements}>;
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr1[i]')} + ${type.to_u32('arr2[i]')} +
                    ${type.to_u32('arr3[i]')} + ${type.to_u32('arr4[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0);
});

g.test('workgroup_array_byte_size').
desc(`Test that arrays in the workgroup address space up to the maximum size are supported.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const maxSize = t.device.limits.maxComputeWorkgroupStorageSize;
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(maxSize / type.size);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    var<workgroup> arr : array<${t.params.type}, ${elements}>;

    @compute @workgroup_size(1)
    fn main() {
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0);
});

g.test('workgroup_array_byte_size_override').
desc(`Test that arrays in the workgroup address space up to the maximum size are supported.`).
params((u) => u.combine('type', keysOf(kArrayElements))).
fn((t) => {
  const maxSize = t.device.limits.maxComputeWorkgroupStorageSize;
  const type = kArrayElements[t.params.type];
  const elements = Math.floor(maxSize / type.size);
  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : array<u32, ${elements}>;

    // Set the default element count far too large, which we later override with a valid value.
    override elements = ${elements} * 1000;
    var<workgroup> arr : array<${t.params.type}, elements>;

    @compute @workgroup_size(1)
    fn main() {
      for (var i = 0; i < ${elements}; i++) {
        buffer[i] = ${type.to_u32('arr[i]')};
      }
    }
    `;

  runShaderTest(t, code, new Uint32Array([...iterRange(elements, (_i) => 0xdeadbeef)]), (_i) => 0, {
    elements
  });
});

g.test('const_array_elements').
desc(`Test that constant array expressions with the maximum number of elements are supported.`).
fn((t) => {
  const type = `array<u32, ${kMaxConstArrayElements}>`;

  let expr = `${type}(`;
  for (let i = 0; i < kMaxConstArrayElements; i++) {
    expr += `${i}, `;
  }
  expr += `)`;

  const code = `
    @group(0) @binding(0) var<storage, read_write> buffer : ${type};

    @compute @workgroup_size(1)
    fn main() {
      buffer = ${expr};
    }
    `;

  runShaderTest(
    t,
    code,
    new Uint32Array([...iterRange(kMaxConstArrayElements, (_i) => 0xdeadbeef)]),
    (i) => i
  );
});