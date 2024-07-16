/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution Tests for structure member accessing expressions
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';
import { Type, u32 } from '../../../../../util/conversion.js';
import { align } from '../../../../../util/math.js';
import { toComparator } from '../../expectation.js';
import { structLayout, structStride } from '../../expression.js';

export const g = makeTestGroup(GPUTest);

const kMemberTypes = [
['bool'],
['u32'],
['vec3f'],
['i32', 'u32'],
['i32', 'f16', 'vec4i', 'mat3x2f'],
['bool', 'u32', 'f16', 'vec3f', 'vec2i'],
['i32', 'u32', 'f32', 'f16', 'vec3f', 'vec4i']];


const kMemberTypesNoBool = kMemberTypes.filter((tys) => !tys.includes('bool'));

async function run(
t,
wgsl,
expected,
input,
inputSource)
{
  const kMinStorageBufferSize = 4;

  const outputBufferSize = Math.max(
    kMinStorageBufferSize,
    structStride(
      expected.map((v) => v.type),
      'storage_rw'
    )
  );

  const outputBuffer = t.createBufferTracked({
    size: outputBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });

  const bindGroupEntries = [
  {
    binding: 0,
    resource: { buffer: outputBuffer }
  }];


  if (input !== null) {
    let inputData;
    if (input instanceof Array) {
      const inputTypes = input.map((v) => v.type);
      const inputBufferSize = Math.max(
        kMinStorageBufferSize,
        structStride(inputTypes, inputSource)
      );
      inputData = new Uint8Array(inputBufferSize);
      structLayout(inputTypes, inputSource, (m) => {
        input[m.index].copyTo(inputData, m.offset);
      });
    } else {
      inputData = new Uint8Array(input);
    }

    const inputBuffer = t.makeBufferWithContents(
      inputData,
      GPUBufferUsage.COPY_SRC | (
      inputSource === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE)
    );

    bindGroupEntries.push({
      binding: 1,
      resource: { buffer: inputBuffer }
    });
  }

  // build the shader module
  const module = t.device.createShaderModule({ code: wgsl });

  // build the pipeline
  const pipeline = await t.device.createComputePipelineAsync({
    layout: 'auto',
    compute: { module, entryPoint: 'main' }
  });

  // build the bind group
  const group = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: bindGroupEntries
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, group);
  pass.dispatchWorkgroups(1);
  pass.end();

  t.queue.submit([encoder.finish()]);

  const checkExpectation = (outputData) => {
    // The list of expectation failures
    const errs = [];

    let offset = 0;
    for (let i = 0; i < expected.length; i++) {
      offset = align(offset, expected[i].type.alignment);
      const got = expected[i].type.read(outputData, offset);
      const cmp = toComparator(expected[i]).compare(got);
      if (!cmp.matched) {
        errs.push(`result ${i}:)
  returned: ${cmp.got}
  expected: ${cmp.expected}`);
      }
      offset += expected[i].type.size;
    }

    return errs.length > 0 ? new Error(errs.join('\n\n')) : undefined;
  };

  t.expectGPUBufferValuesPassCheck(outputBuffer, checkExpectation, {
    type: Uint8Array,
    typedLength: outputBufferSize
  });
}

g.test('buffer').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a value structure in a storage or uniform buffer`).
params((u) =>
u.
combine('member_types', kMemberTypesNoBool).
combine('inputSource', ['uniform', 'storage']).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected = values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};
@group(0) @binding(1) var<${t.params.inputSource}> input : MyStruct;

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

@workgroup_size(1) @compute
fn main() {
  output = input.member_${t.params.member_index};
}
`,
    /* expected */[expected],
    /* input */values,
    /* inputSource */t.params.inputSource === 'uniform' ? 'uniform' : 'storage_r'
  );
});

g.test('buffer_align').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(
  `Test accessing of a value structure in a storage buffer that has members using the @align attribute`
).
params((u) =>
u.
beginSubcases().
combine('member_index', [0, 1, 2]).
combine('alignments', [
[1, 1, 1],
[4, 4, 4],
[4, 8, 16],
[8, 4, 16],
[8, 16, 4]]
)
).
fn(async (t) => {
  const memberTypes = ['i32', 'u32', 'f32'];
  const values = memberTypes.map((ty, i) => Type[ty].create(i));
  const expected = values[t.params.member_index];
  const input = new Uint8Array(64);
  let offset = 4; // pre : i32
  for (let i = 0; i < 3; i++) {
    offset = align(offset, t.params.alignments[i]);
    values[i].copyTo(input, offset);
    offset += values[i].type.size;
  }
  await run(
    t,
    `
@group(0) @binding(0) var<storage, read_write> output : ${expected.type};
@group(0) @binding(1) var<storage> input : MyStruct;

struct MyStruct {
  pre : i32,
  @align(${t.params.alignments[0]}) member_0 : ${memberTypes[0]},
  @align(${t.params.alignments[1]}) member_1 : ${memberTypes[1]},
  @align(${t.params.alignments[2]}) member_2 : ${memberTypes[2]},
  post : i32,
};

@workgroup_size(1) @compute
fn main() {
output = input.member_${t.params.member_index};
}
`,
    /* expected */[expected],
    /* input */input,
    /* inputSource */'storage_r'
  );
});

g.test('buffer_size').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(
  `Test accessing of a value structure in a storage buffer that has members using the @size attribute`
).
params((u) =>
u.
beginSubcases().
combine('member_index', [0, 1, 2]).
combine('sizes', [
[4, 4, 4],
[4, 8, 16],
[8, 4, 16],
[8, 16, 4]]
)
).
fn(async (t) => {
  const memberTypes = ['i32', 'u32', 'f32'];
  const values = memberTypes.map((ty, i) => Type[ty].create(i));
  const expected = values[t.params.member_index];
  const input = new Uint8Array(64);
  let offset = 4; // pre : i32
  for (let i = 0; i < 3; i++) {
    offset = align(offset, values[i].type.alignment);
    values[i].copyTo(input, offset);
    offset += t.params.sizes[i];
  }
  await run(
    t,
    `
@group(0) @binding(0) var<storage, read_write> output : ${expected.type};
@group(0) @binding(1) var<storage> input : MyStruct;

struct MyStruct {
  pre : i32,
  @size(${t.params.sizes[0]}) member_0 : ${memberTypes[0]},
  @size(${t.params.sizes[1]}) member_1 : ${memberTypes[1]},
  @size(${t.params.sizes[2]}) member_2 : ${memberTypes[2]},
  post : i32,
};

@workgroup_size(1) @compute
fn main() {
output = input.member_${t.params.member_index};
}
`,
    /* expected */[expected],
    /* input */input,
    /* inputSource */'storage_r'
  );
});

g.test('buffer_pointer').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a value structure via a pointer to a storage or uniform buffer`).
params((u) =>
u.
combine('member_types', kMemberTypesNoBool).
combine('inputSource', ['uniform', 'storage']).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected = values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};
@group(0) @binding(1) var<${t.params.inputSource}> input : MyStruct;

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

@workgroup_size(1) @compute
fn main() {
  let ptr = &input;
  output = (*ptr).member_${t.params.member_index};
}
`,
    /* expected */[expected],
    /* input */values,
    /* inputSource */t.params.inputSource === 'uniform' ? 'uniform' : 'storage_r'
  );
});

g.test('let').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a let structure`).
params((u) =>
u.
combine('member_types', kMemberTypes).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected =
  memberType === Type.bool ?
  u32(values[t.params.member_index].value === true ? 1 : 0) :
  values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

@workgroup_size(1) @compute
fn main() {
  let s = MyStruct(${values.map((v) => v.wgsl()).join(', ')});
  let v = s.member_${t.params.member_index};
  output = ${memberType === Type.bool ? `select(0u, 1u, v)` : 'v'};
}
`,
    /* expected */[expected],
    /* input */null,
    /* inputSource */'const'
  );
});

g.test('param').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a parameter structure`).
params((u) =>
u.
combine('member_types', kMemberTypes).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected =
  memberType === Type.bool ?
  u32(values[t.params.member_index].value === true ? 1 : 0) :
  values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

fn f(s : MyStruct) -> ${t.params.member_types[t.params.member_index]} {
  return s.member_${t.params.member_index};
}

@workgroup_size(1) @compute
fn main() {
  let v = f(MyStruct(${values.map((v) => v.wgsl()).join(', ')}));
  output = ${memberType === Type.bool ? `select(0u, 1u, v)` : 'v'};
}
`,
    /* expected */[expected],
    /* input */null,
    /* inputSource */'const'
  );
});

g.test('const').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a const value structure`).
params((u) =>
u.
combine('member_types', kMemberTypes).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected =
  memberType === Type.bool ?
  u32(values[t.params.member_index].value === true ? 1 : 0) :
  values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

const S = MyStruct(${values.map((v) => v.wgsl()).join(', ')});

@workgroup_size(1) @compute
fn main() {
  let v = S.member_${t.params.member_index};
  output = ${memberType === Type.bool ? `select(0u, 1u, v)` : 'v'};
}
`,
    /* expected */[expected],
    /* input */null,
    /* inputSource */'const'
  );
});

g.test('const_nested').
specURL('https://www.w3.org/TR/WGSL/#struct-access-expr').
desc(`Test accessing of a const value structure nested in another structure`).
params((u) =>
u.
combine('member_types', kMemberTypes).
beginSubcases().
expand('member_index', (t) => t.member_types.map((_, i) => i))
).
beforeAllSubcases((t) => {
  if (t.params.member_types.includes('f16')) {
    t.selectDeviceOrSkipTestCase('shader-f16');
  }
}).
fn(async (t) => {
  const memberType = Type[t.params.member_types[t.params.member_index]];
  const values = t.params.member_types.map((ty, i) => Type[ty].create(i));
  const expected =
  memberType === Type.bool ?
  u32(values[t.params.member_index].value === true ? 1 : 0) :
  values[t.params.member_index];

  await run(
    t,
    `
${t.params.member_types.includes('f16') ? 'enable f16;' : ''}

@group(0) @binding(0) var<storage, read_write> output : ${expected.type};

struct MyStruct {
  ${t.params.member_types.map((ty, i) => `  member_${i} : ${ty},`).join('\n')}
};

struct Outer {
  pre : i32,
  inner : MyStruct,
  post : i32,
}

const S = Outer(10, MyStruct(${values.map((v) => v.wgsl()).join(', ')}), 20);

@workgroup_size(1) @compute
fn main() {
  let v = S.inner.member_${t.params.member_index};
  output = ${memberType === Type.bool ? `select(0u, 1u, v)` : 'v'};
}
`,
    /* expected */[expected],
    /* input */null,
    /* inputSource */'const'
  );
});