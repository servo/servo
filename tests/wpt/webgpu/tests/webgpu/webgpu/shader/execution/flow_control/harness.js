/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { Colors } from '../../../../common/util/colors.js';
/**
 * Options for runFlowControlTest()
 */












/**
 * The builder interface for the runFlowControlTest() callback.
 * This interface is indented to be used to inject WGSL logic into the test
 * shader.
 * @see runFlowControlTest
 */
























/**
 * Builds, runs then checks the output of a flow control shader test.
 *
 * `build_wgsl` is a function that's called to build the WGSL shader.
 * This function takes a FlowControlTestBuilder as the single argument, and
 * returns either a string which is embedded into the WGSL entrypoint function,
 * or an object of the signature `{ entrypoint: string; extra: string }` which
 * contains the entrypoint code, along with additional module-scope code.
 *
 * The FlowControlTestBuilder should be used to insert expectations into WGSL to
 * validate control flow. FlowControlTestBuilder also can be used to add values
 * to the shader which cannot be optimized away.
 *
 * Example, testing that an if-statement behaves as expected:
 *
 * ```
 *   runFlowControlTest(t, f =>
 *   `
 *    ${f.expect_order(0)}
 *    if (${f.value(true)}) {
 *      ${f.expect_order(1)}
 *    } else {
 *      ${f.expect_not_reached()}
 *    }
 *    ${f.expect_order(2)}
 *  `);
 * ```
 *
 * @param t The test object
 * @param builder The shader builder function that takes a
 * FlowControlTestBuilder as the single argument, and returns either a WGSL
 * string which is embedded into the WGSL entrypoint function, or a structure
 * with entrypoint-scoped WGSL code and extra module-scope WGSL code.
 */
export function runFlowControlTest(
t,
build_wgsl)
{
  const inputData = new Array();












  const expectations = new Array();

  const build_wgsl_result = build_wgsl({
    value: (v) => {
      if (t.params.preventValueOptimizations) {
        if (typeof v === 'boolean') {
          inputData.push(v ? 1 : 0);
          return `inputs[${inputData.length - 1}] != 0`;
        }
        inputData.push(v);
        return `inputs[${inputData.length - 1}]`;
      } else {
        return `${v}`;
      }
    },
    expect_order: (...expected) => {
      expectations.push({
        kind: 'events',
        stack: Error().stack,
        values: expected,
        counter: 0
      });
      // Expectation id starts from 1 to distinguish from initialization 0.
      return `push_output(${expectations.length}); // expect_order(${expected.join(', ')})`;
    },
    expect_not_reached: () => {
      expectations.push({
        kind: 'not-reached',
        stack: Error().stack
      });
      // Expectation id starts from 1 to distinguish from initialization 0.
      return `push_output(${expectations.length}); // expect_not_reached()`;
    }
  });

  const built_wgsl =
  typeof build_wgsl_result === 'string' ?
  { entrypoint: build_wgsl_result, extra: '' } :
  build_wgsl_result;

  const main_wgsl = built_wgsl.entrypoint !== undefined ? built_wgsl : built_wgsl.entrypoint;

  const wgsl = `
struct Outputs {
  count : u32,
  data  : array<u32>,
};
@group(0) @binding(0) var<storage, read>       inputs  : array<i32>;
@group(0) @binding(1) var<storage, read_write> outputs : Outputs;

fn push_output(value : u32) {
  outputs.data[outputs.count] = value;
  outputs.count++;
}

@compute @workgroup_size(1)
fn main() {
  _ = &inputs;
  _ = &outputs;
  ${main_wgsl.entrypoint}
}
${main_wgsl.extra}
`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  // If there are no inputs, just put a single value in the buffer to keep
  // makeBufferWithContents() happy.
  if (inputData.length === 0) {
    inputData.push(0);
  }

  const inputBuffer = t.makeBufferWithContents(new Uint32Array(inputData), GPUBufferUsage.STORAGE);

  const maxOutputValues = 1000;
  const outputBuffer = t.createBufferTracked({
    size: 4 * (1 + maxOutputValues),
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: inputBuffer } },
    { binding: 1, resource: { buffer: outputBuffer } }]

  });

  // Run the shader.
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.eventualExpectOK(
    t.
    readGPUBufferRangeTyped(outputBuffer, {
      type: Uint32Array,
      typedLength: outputBuffer.size / 4
    }).
    then((outputs) => {
      // outputs[0]    is the number of outputted values
      // outputs[1..N] holds the outputted values
      const outputCount = outputs.data[0];
      if (outputCount > maxOutputValues) {
        return new Error(
          `output data count (${outputCount}) exceeds limit of ${maxOutputValues}`
        );
      }

      // returns an Error with the given message and WGSL source
      const fail = (err) => Error(`${err}\nWGSL:\n${Colors.dim(Colors.blue(wgsl))}`);

      // returns a string that shows the outputted values to help understand the whole trace.
      const print_output_value = () => {
        const subarray = outputs.data.subarray(1, outputCount + 1);
        return `Output values (length: ${outputCount}): ${subarray.join(', ')}`;
      };

      // returns a colorized string of the expect_order() call, highlighting
      // the event number that caused an error.
      const expect_order_err = (expectation, err_idx) => {
        let out = 'expect_order(';
        for (let i = 0; i < expectation.values.length; i++) {
          if (i > 0) {
            out += ', ';
          }
          if (i < err_idx) {
            out += Colors.green(`${expectation.values[i]}`);
          } else if (i > err_idx) {
            out += Colors.dim(`${expectation.values[i]}`);
          } else {
            out += Colors.red(`${expectation.values[i]}`);
          }
        }
        out += ')';
        return out;
      };

      // Each of the outputted values represents an event
      // Check that each event is as expected
      for (let event = 0; event < outputCount; event++) {
        const eventValue = outputs.data[1 + event]; // outputs.data[0] is count
        // Expectation id starts from 1, and 0 is invalid value.
        if (eventValue === 0) {
          return fail(
            `outputs.data[${event}] is initial value 0, doesn't refer to any valid expectations)\n${print_output_value()}`
          );
        }
        const expectationIndex = eventValue - 1;
        if (expectationIndex >= expectations.length) {
          return fail(
            `outputs.data[${event}] value (${expectationIndex}) exceeds number of expectations (${
            expectations.length
            })\n${print_output_value()}`
          );
        }
        const expectation = expectations[expectationIndex];
        switch (expectation.kind) {
          case 'not-reached':
            return fail(
              `expect_not_reached() reached at event ${event}\n${print_output_value()}\n${
              expectation.stack
              }`
            );
          case 'events':
            if (expectation.counter >= expectation.values.length) {
              return fail(
                `${expect_order_err(
                  expectation,
                  expectation.counter
                )}) unexpectedly reached at event ${Colors.red(
                  `${event}`
                )}\n${print_output_value()}\n${expectation.stack}`
              );
            }
            if (event !== expectation.values[expectation.counter]) {
              return fail(
                `${expect_order_err(expectation, expectation.counter)} expected event ${
                expectation.values[expectation.counter]
                }, got ${event}\n${print_output_value()}\n${expectation.stack}`
              );
            }

            expectation.counter++;
            break;
        }
      }

      // Finally check that all expect_order() calls were reached
      for (const expectation of expectations) {
        if (expectation.kind === 'events' && expectation.counter !== expectation.values.length) {
          return fail(
            `${expect_order_err(expectation, expectation.counter)} event ${
            expectation.values[expectation.counter]
            } was not reached\n${expectation.stack}\n${print_output_value()}`
          );
        }
      }
      outputs.cleanup();
      return undefined;
    })
  );
}