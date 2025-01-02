/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for operator precedence.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// The list of test cases and their expected results.




const kExpressions = {
  add_mul: { expr: 'kThree + kSeven * kEleven', result: 80 },
  mul_add: { expr: 'kThree * kSeven + kEleven', result: 32 },
  sub_neg: { expr: 'kThree - - kSeven', result: 10 },
  neg_shl: { expr: '- kThree << u32(kSeven)', result: -384 },
  neg_shr: { expr: '- kThree >> u32(kSeven)', result: -1 },
  neg_add: { expr: '- kThree + kSeven', result: 4 },
  neg_mul: { expr: '- kThree * kSeven', result: -21 },
  neg_and: { expr: '- kThree & kSeven', result: 5 },
  neg_or: { expr: '- kThree | kSeven', result: -1 },
  neg_xor: { expr: '- kThree ^ kSeven', result: -6 },
  comp_add: { expr: '~ kThree + kSeven', result: 3 },
  mul_deref: { expr: 'kThree * * ptr_five', result: 15 },
  not_and: { expr: 'i32(! kFalse && kFalse)', result: 0 },
  not_or: { expr: 'i32(! kTrue || kTrue)', result: 1 },
  eq_and: { expr: 'i32(kFalse == kTrue && kFalse)', result: 0 },
  and_eq: { expr: 'i32(kFalse && kTrue == kFalse)', result: 0 },
  eq_or: { expr: 'i32(kFalse == kFalse || kTrue)', result: 1 },
  or_eq: { expr: 'i32(kTrue || kFalse == kFalse)', result: 1 },
  add_swizzle: { expr: '(vec + vec . y) . z', result: 8 }
};

g.test('precedence').
desc(
  `
    Test that operator precedence rules are correctly implemented.
    `
).
params((u) =>
u.
combine('expr', keysOf(kExpressions)).
combine('decl', ['literal', 'const', 'override', 'var<private>']).
combine('strip_spaces', [false, true])
).
fn((t) => {
  const expr = kExpressions[t.params.expr];

  let decl = t.params.decl;
  let expr_wgsl = expr.expr;
  if (t.params.decl === 'literal') {
    decl = 'const';
    expr_wgsl = expr_wgsl.replace(/kThree/g, '3');
    expr_wgsl = expr_wgsl.replace(/kSeven/g, '7');
    expr_wgsl = expr_wgsl.replace(/kEleven/g, '11');
    expr_wgsl = expr_wgsl.replace(/kFalse/g, 'false');
    expr_wgsl = expr_wgsl.replace(/kTrue/g, 'true');
  }
  if (t.params.strip_spaces) {
    expr_wgsl = expr_wgsl.replace(/ /g, '');
  }
  const wgsl = `
      @group(0) @binding(0) var<storage, read_write> buffer : i32;

      ${decl} kFalse = false;
      ${decl} kTrue = true;

      ${decl} kThree = 3;
      ${decl} kSeven = 7;
      ${decl} kEleven = 11;

      @compute @workgroup_size(1)
      fn main() {
        var five = 5;
        var vec = vec4(1, kThree, 5, kSeven);
        let ptr_five = &five;

        buffer = ${expr_wgsl};
      }
    `;
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl })
    }
  });

  // Allocate a buffer and fill it with 0xdeadbeef.
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([0xdeadbeef]),
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

  // Check that the result is as expected.
  t.expectGPUBufferValuesEqual(outputBuffer, new Int32Array([expr.result]));
});