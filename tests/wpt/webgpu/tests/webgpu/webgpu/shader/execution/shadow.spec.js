/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Execution Tests for shadowing
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { iterRange } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

/**
 * Run a shader and check that the buffer output matches expectations.
 *
 * @param t The test object
 * @param wgsl The shader source
 * @param expected The array of expected values after running the shader
 */
function runShaderTest(t, wgsl, expected) {
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main',
    },
  });

  // Allocate a buffer and fill it with 0xdeadbeef words.
  const outputBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(expected.length, x => 0xdeadbeef)]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  );

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [{ binding: 0, resource: { buffer: outputBuffer } }],
  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  // Check that only the non-padding bytes were modified.
  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}

g.test('declaration')
  .desc(`Test that shadowing is handled correctly`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_var_start: u32,
        my_var_block_shadow: u32,
        my_var_unshadow: u32,
        my_var_param_shadow: u32,
        my_var_param_reshadow: u32,
        my_var_after_func: u32,

        my_const_start: u32,
        my_const_block_shadow: u32,
        my_const_unshadow: u32,
        my_const_param_shadow: u32,
        my_const_param_reshadow: u32,
        my_const_after_func: u32,

        my_let_block_shadow: u32,
        my_let_param_reshadow: u32,
        my_let_after_func: u32,

        my_func_param_shadow: u32,
        my_func_shadow: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      var<private> my_var: u32  = 1;
      const my_const: u32 = 100;

      @compute @workgroup_size(1)
      fn main() {
        let my_let = 200u;

        buffer.my_var_start = my_var;  // 1
        buffer.my_const_start = my_const;  // 100

        {
            var my_var: u32 = 10;
            const my_const: u32 = 110;

            buffer.my_var_block_shadow = my_var;  // 10
            buffer.my_const_block_shadow = my_const;  // 110

            let my_let = 210u;
            buffer.my_let_block_shadow = my_let;  // 210
        }

        buffer.my_var_unshadow = my_var;  // 1
        buffer.my_const_unshadow = my_const;  // 100

        my_func(20, 120, my_let, 300);

        buffer.my_var_after_func = my_var;  // 1
        buffer.my_const_after_func = my_const;  // 100
        buffer.my_let_after_func = my_let;  // 200;
      };

      // Note, defined after |main|
      fn my_func(my_var: u32, my_const: u32, my_let: u32, my_func: u32) {
        buffer.my_var_param_shadow = my_var;  // 20
        buffer.my_const_param_shadow = my_const;  // 120

        buffer.my_func_param_shadow = my_func; // 300

        // Need block here because of scoping rules for parameters
        {
          var my_var = 30u;
          const my_const = 130u;

          buffer.my_var_param_reshadow = my_var; // 30
          buffer.my_const_param_reshadow = my_const; // 130

          let my_let = 220u;
          buffer.my_let_param_reshadow = my_let; // 220

          let my_func: u32 = 310;
          buffer.my_func_shadow = my_func;  // 310
        }
      }
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        // my_var
        1, // my_var_start
        10, // my_var_block_shadow
        1, // my_var_unshadow
        20, // my_var_param_shadow
        30, // my_var_param_reshadow
        1, // my_var_after_func
        // my_const
        100, // my_const_start
        110, // my_const_block_shadow
        100, // my_const_unshadow
        120, // my_const_param_shadow
        130, // my_const_param_reshadow
        100, // my_const_after_func
        // my_let
        210, // my_let_block_shadow
        220, // my_let_param_reshadow
        200, // my_let_after_func
        // my_func
        300, // my_func_param_shadow
        310, // my_func_shadow
      ])
    );
  });

g.test('builtin')
  .desc(`Test that shadowing a builtin name is handled correctly`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_max_shadow: u32,
        max_call: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        let max = 400u;
        buffer.my_max_shadow = max;

        my_func();
      };

      fn my_func() {
        buffer.max_call = max(310u, 410u);
      }
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        // my_max
        400, // my_max_shadow
        410, // max_call
      ])
    );
  });

g.test('for_loop')
  .desc(`Test that shadowing is handled correctly with for loops`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_idx_before: u32,
        my_idx_loop: array<u32, 2>,
        my_idx_after: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        var my_idx = 500u;
        buffer.my_idx_before = my_idx; // 500;
        for (var my_idx = 0u; my_idx < 2u; my_idx++) {
          let pos = my_idx;
          var my_idx = 501u + my_idx;
          buffer.my_idx_loop[pos] = my_idx;  // 501, 502
        }
        buffer.my_idx_after = my_idx; // 500;
      };
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        500, // my_idx_before
        501, // my_idx_loop[0]
        502, // my_idx_loop[1]
        500, // my_idx_after
      ])
    );
  });

g.test('while')
  .desc(`Test that shadowing is handled correctly with while loops`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_idx_before: u32,
        my_idx_loop: array<u32, 2>,
        my_idx_after: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        var my_idx = 0u;
        buffer.my_idx_before = my_idx; // 0;

        var counter = 0u;
        while (counter < 2) {
          var my_idx = 500u + counter;
          buffer.my_idx_loop[counter] = my_idx;  // 500, 501

          counter += 1;
        }

        buffer.my_idx_after = my_idx; // 1;
      };
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        0, // my_idx_before
        500, // my_idx_loop[0]
        501, // my_idx_loop[1]
        0, // my_idx_after
      ])
    );
  });

g.test('loop')
  .desc(`Test that shadowing is handled correctly with loops`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_idx_before: u32,
        my_idx_loop: array<u32, 2>,
        my_idx_continuing: array<u32, 2>,
        my_idx_after: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        var my_idx = 0u;
        buffer.my_idx_before = my_idx; // 0;

        var counter = 0u;
        loop {
          var my_idx = 500u + counter;
          buffer.my_idx_loop[counter] = my_idx;  // 500, 501


          continuing {
            var my_idx = 600u + counter;
            buffer.my_idx_continuing[counter] = my_idx; // 600, 601

            counter += 1;
            break if counter == 2;
          }
        }
        buffer.my_idx_after = my_idx; // 1;
      };
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        0, // my_idx_before
        500, // my_idx_loop[0]
        501, // my_idx_loop[1]
        600, // my_idx_continuing[0]
        601, // my_idx_continuing[1]
        0, // my_idx_after
      ])
    );
  });

g.test('switch')
  .desc(`Test that shadowing is handled correctly with a switch`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_idx_before: u32,
        my_idx_case: u32,
        my_idx_default: u32,
        my_idx_after: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        var my_idx = 0u;
        buffer.my_idx_before = my_idx; // 0;

        for (var i = 0; i < 2; i++) {
          switch (i) {
            case 0: {
              var my_idx = 10u;
              buffer.my_idx_case = my_idx; // 10
            }
            default: {
              var my_idx = 20u;
              buffer.my_idx_default = my_idx; // 20
            }
          }
        }

        buffer.my_idx_after = my_idx; // 1;
      };
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        0, // my_idx_before
        10, // my_idx_case
        20, // my_idx_default
        0, // my_idx_after
      ])
    );
  });

g.test('if')
  .desc(`Test that shadowing is handled correctly with a switch`)
  .fn(t => {
    const wgsl = `
      struct S {
        my_idx_before: u32,
        my_idx_if: u32,
        my_idx_elseif: u32,
        my_idx_else: u32,
        my_idx_after: u32,
      }
      @group(0) @binding(0) var<storage, read_write> buffer : S;

      @compute @workgroup_size(1)
      fn main() {
        var my_idx = 0u;
        buffer.my_idx_before = my_idx; // 0;

        for (var i = 0; i < 3; i++) {
          if i == 0 {
            var my_idx = 10u;
            buffer.my_idx_if = my_idx; // 10
          } else if i == 1 {
            var my_idx = 20u;
            buffer.my_idx_elseif = my_idx; // 20
          } else {
            var my_idx = 30u;
            buffer.my_idx_else = my_idx; // 30
          }
        }

        buffer.my_idx_after = my_idx; // 1;
      };
    `;
    runShaderTest(
      t,
      wgsl,
      new Uint32Array([
        0, // my_idx_before
        10, // my_idx_if
        20, // my_idx_elseif
        30, // my_idx_else
        0, // my_idx_after
      ])
    );
  });
