/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Execution tests for discard.

The discard statement converts invocations into helpers.
This results in the following conditions:
  * No outputs are written
  * No resources are written
  * Atomics are undefined

Conditions that still occur:
  * Derivative calculations are correct
  * Reads
  * Writes to non-external memory
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { iterRange } from '../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import { checkElementsPassPredicate } from '../../../util/check_contents.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

// Framebuffer dimensions
const kWidth = 64;
const kHeight = 64;

const kSharedCode = `
@group(0) @binding(0) var<storage, read_write> output: array<vec2f>;
@group(0) @binding(1) var<storage, read_write> atomicIndex : atomic<u32>;
@group(0) @binding(2) var<uniform> uniformValues : array<vec4u, 5>;

@vertex
fn vsMain(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
  const vertices = array(
    vec2(-1, -1), vec2(-1,  0), vec2( 0, -1),
    vec2(-1,  0), vec2( 0,  0), vec2( 0, -1),

    vec2( 0, -1), vec2( 0,  0), vec2( 1, -1),
    vec2( 0,  0), vec2( 1,  0), vec2( 1, -1),

    vec2(-1,  0), vec2(-1,  1), vec2( 0,  0),
    vec2(-1,  1), vec2( 0,  1), vec2( 0,  0),

    vec2( 0,  0), vec2( 0,  1), vec2( 1,  0),
    vec2( 0,  1), vec2( 1,  1), vec2( 1,  0),
  );
  return vec4f(vec2f(vertices[index]), 0, 1);
}
`;

function drawFullScreen(
t,
code,
useStorageBuffers,
dataChecker,
framebufferChecker)
{
  t.skipIf(
    useStorageBuffers &&
    t.isCompatibility &&
    !(t.device.limits.maxStorageBuffersInFragmentStage >= 2),
    `maxStorageBuffersInFragmentStage${t.device.limits.maxStorageBuffersInFragmentStage} is less than 2`
  );

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({ code }),
      entryPoint: 'vsMain'
    },
    fragment: {
      module: t.device.createShaderModule({ code }),
      entryPoint: 'fsMain',
      targets: [{ format: 'rg32uint' }]
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const bytesPerWord = 4;
  const framebuffer = t.createTextureTracked({
    size: [kWidth, kHeight],
    usage:
    GPUTextureUsage.COPY_SRC |
    GPUTextureUsage.COPY_DST |
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING,
    format: 'rg32uint'
  });

  // Create a buffer to copy the framebuffer contents into.
  // Initialize with a sentinel value and load this buffer to detect unintended writes.
  const fbBuffer = t.makeBufferWithContents(
    new Uint32Array([...iterRange(kWidth * kHeight * 2, (x) => kWidth * kHeight)]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );

  // Create a buffer to hold the storage shader resources.
  // (0,0) = vec2u width * height
  // (0,1) = u32
  const dataSize = 2 * kWidth * kHeight * bytesPerWord;
  const dataBufferSize = dataSize + bytesPerWord;
  const dataBuffer = t.createBufferTracked({
    size: dataBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });

  const uniformSize = bytesPerWord * 5 * 4;
  const uniformBuffer = t.makeBufferWithContents(
    // Loop bound, [derivative constants].

    new Uint32Array([
    4, 0, 0, 0,
    1, 0, 0, 0,
    4, 0, 0, 0,
    4, 0, 0, 0,
    7, 0, 0, 0]
    ),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  );

  // 'atomicIndex' packed at the end of the buffer.
  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    ...(useStorageBuffers ?
    [
    {
      binding: 0,
      resource: {
        buffer: dataBuffer,
        offset: 0,
        size: dataSize
      }
    },
    {
      binding: 1,
      resource: {
        buffer: dataBuffer,
        offset: dataSize,
        size: bytesPerWord
      }
    }] :

    []),
    {
      binding: 2,
      resource: {
        buffer: uniformBuffer,
        offset: 0,
        size: uniformSize
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture(
    {
      buffer: fbBuffer,
      offset: 0,
      bytesPerRow: kWidth * bytesPerWord * 2,
      rowsPerImage: kHeight
    },
    { texture: framebuffer },
    { width: kWidth, height: kHeight }
  );
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: framebuffer.createView(),
      loadOp: 'load',
      storeOp: 'store'
    }]

  });
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.draw(24);
  pass.end();
  encoder.copyTextureToBuffer(
    { texture: framebuffer },
    {
      buffer: fbBuffer,
      offset: 0,
      bytesPerRow: kWidth * bytesPerWord * 2,
      rowsPerImage: kHeight
    },
    { width: kWidth, height: kHeight }
  );
  t.queue.submit([encoder.finish()]);

  if (useStorageBuffers) {
    t.expectGPUBufferValuesPassCheck(dataBuffer, dataChecker, {
      type: Float32Array,
      typedLength: dataSize / bytesPerWord
    });
  }

  t.expectGPUBufferValuesPassCheck(fbBuffer, framebufferChecker, {
    type: Uint32Array,
    typedLength: kWidth * kHeight * 2
  });
}

g.test('all').
desc('Test a shader that discards all fragments').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  _ = uniformValues[0];
  discard;
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  ` :
  ''
  }
  return vec2u(1);
}
`;

  // No storage writes occur.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === 0;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            return 0;
          }
        }]

      }
    );
  };

  // No fragment outputs occur.
  const fbChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === kWidth * kHeight;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            return 0;
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('three_quarters').
desc('Test a shader that discards all but the upper-left quadrant fragments').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  _ = uniformValues[0];
  if (pos.x >= 0.5 * ${kWidth} || pos.y >= 0.5 * ${kHeight}) {
    discard;
  }
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  return vec2u(idx);
  ` :
  `
  return vec2(u32(pos.x), u32(pos.y));

  `
  }
}
`;

  // Only the the upper left quadrant is kept.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        const is_x = idx % 2 === 0;
        if (is_x) {
          return value < 0.5 * kWidth;
        } else {
          return value < 0.5 * kHeight;
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            const is_x = idx % 2 === 0;
            if (is_x) {
              const x = Math.floor(idx / 2) % kWidth;
              if (x >= kWidth / 2) {
                return 0;
              }
            } else {
              const y = Math.floor((idx - 1) / kWidth);
              if (y >= kHeight / 2) {
                return 0;
              }
            }
            if (is_x) {
              return `< ${0.5 * kWidth}`;
            } else {
              return `< ${0.5 * kHeight}`;
            }
          }
        }]

      }
    );
  };
  const fbChecker = (a) => {
    const discarded = (x, y) => x >= kWidth / 2 || y >= kHeight / 2;
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        const fragId = idx / 2 | 0;
        const x = fragId % kWidth;
        const y = fragId / kWidth | 0;
        if (discarded(x, y)) {
          return value === kWidth * kHeight;
        } else {
          if (useStorageBuffers) {
            return value < kWidth * kHeight / 4;
          } else {
            return value === (idx % 2 ? y : x);
          }
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            const x = idx % kWidth;
            const y = Math.floor(idx / kWidth);
            if (discarded(x, y)) {
              return 0;
            } else {
              return useStorageBuffers ? 'any' : idx % 2 ? y : x;
            }
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('function_call').
desc('Test discards happening in a function call').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

fn foo(pos : vec2f) {
  let p = vec2i(pos);
  if p.x <= ${kWidth} / 2 && p.y <= ${kHeight} / 2 {
    discard;
  }
  if p.x >= ${kWidth} / 2 && p.y >= ${kHeight} / 2 {
    discard;
  }
}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  _ = uniformValues[0];
  foo(pos.xy);
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  return vec2u(idx);
  ` :
  `
  return vec2u(u32(pos.x), u32(pos.y));
  `
  }
}
`;

  // Only the upper right and bottom left quadrants are kept.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        const is_x = idx % 2 === 0;
        if (value === 0.0) {
          return is_x ? a[idx + 1] === 0 : a[idx - 1] === 0;
        }

        let expect = is_x ? kWidth : kHeight;
        expect = 0.5 * expect;
        if (value < expect) {
          return is_x ? a[idx + 1] > 0.5 * kWidth : a[idx - 1] > 0.5 * kHeight;
        } else {
          return is_x ? a[idx + 1] < 0.5 * kWidth : a[idx - 1] < 0.5 * kHeight;
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            if (idx < kWidth * kHeight / 2) {
              return 'any';
            } else {
              return 0;
            }
          }
        }]

      }
    );
  };
  const fbChecker = (a) => {
    const discarded = (x, y) =>
    x >= kWidth / 2 && y >= kHeight / 2 || x <= kWidth / 2 && y <= kHeight / 2;
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        const fragId = idx / 2 | 0;
        const x = fragId % kWidth;
        const y = fragId / kWidth | 0;
        if (discarded(x, y)) {
          return value === kWidth * kHeight;
        } else {
          if (useStorageBuffers) {
            return value < kWidth * kHeight / 2;
          } else {
            return value === (idx % 2 ? y : x);
          }
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            const x = idx % kWidth;
            const y = Math.floor(idx / kWidth);
            if (discarded(x, y)) {
              return kWidth * kHeight;
            }
            return useStorageBuffers ? 'any' : idx % 2 ? y : x;
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('loop').
desc('Test discards in a loop').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  _ = uniformValues[0];
  for (var i = 0; i < 2; i++) {
    if i > 0 {
      discard;
    }
  }
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  ` :
  ''
  }
  return vec2u(1);
}
`;

  // No storage writes occur.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === 0;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            return 0;
          }
        }]

      }
    );
  };

  // No fragment outputs occur.
  const fbChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === kWidth * kHeight;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            return kWidth * kHeight;
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('continuing').
desc('Test discards in a loop').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  _ = uniformValues[0];
  var i = 0;
  loop {
    continuing {
      if i > 0 {
        discard;
      }
      i++;
      break if i >= 2;
    }
  }
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  ` :
  ''
  }
  return vec2u(1);
}
`;

  // No storage writes occur.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === 0;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            return 0;
          }
        }]

      }
    );
  };

  // No fragment outputs occur.
  const fbChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === kWidth * kHeight;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            return kWidth * kHeight;
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('uniform_read_loop').
desc('Test that helpers read a uniform value in a loop').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  discard;
  for (var i = 0u; i < uniformValues[0].x; i++) {
  }
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = pos.xy;
  ` :
  ''
  }
  return vec2u(1);
}
`;

  // No storage writes occur.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === 0;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            return 0;
          }
        }]

      }
    );
  };

  // No fragment outputs occur.
  const fbChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        return value === kWidth * kHeight;
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            return kWidth * kHeight;
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});

g.test('derivatives').
desc('Test that derivatives are correct in the presence of discard').
params((u) => u.combine('useStorageBuffers', [false, true])).
fn((t) => {
  const { useStorageBuffers } = t.params;
  const code = `
${kSharedCode}

@fragment
fn fsMain(@builtin(position) pos : vec4f) -> @location(0) vec2u {
  let ipos = vec2i(pos.xy);
  let lsb = ipos & vec2(0x1);
  let left_sel = select(2, 4, lsb.y == 1);
  let right_sel = select(1, 3, lsb.y == 1);
  let uidx = select(left_sel, right_sel, lsb.x == 1);
  if ((lsb.x | lsb.y) & 0x1) == 0 {
    discard;
  }

  let v = uniformValues[uidx].x;
  let dx = dpdx(f32(v));
  let dy = dpdy(f32(v));
  ${
  useStorageBuffers ?
  `
  let idx = atomicAdd(&atomicIndex, 1);
  output[idx] = vec2(dx, dy);
  return vec2u(idx);
    ` :
  `
  return bitcast<vec2u>(vec2f(dx, dy));
    `
  }
}
`;

  // One pixel per quad is discarded. The derivatives values are always the same +/- 3.
  const dataChecker = (a) => {
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        if (idx < 3 * (2 * kWidth * kHeight) / 4) {
          return value === -3 || value === 3;
        } else {
          return value === 0;
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'data exp ==',
          getValueForCell: (idx) => {
            if (idx < 3 * (2 * kWidth * kHeight) / 4) {
              return '+/- 3';
            } else {
              return 0;
            }
          }
        }]

      }
    );
  };

  // 3/4 of the fragments are written.
  const fbChecker = (a) => {
    const discarded = (x, y) => ((x | y) & 0x1) === 0;
    const asF32 = new Float32Array(1);
    const asU32 = new Uint32Array(asF32.buffer);
    return checkElementsPassPredicate(
      a,
      (idx, value) => {
        const fragId = idx / 2 | 0;
        const x = fragId % kWidth;
        const y = fragId / kWidth | 0;
        if (discarded(x, y)) {
          return value === kWidth * kHeight;
        } else {
          if (useStorageBuffers) {
            return value < 3 * (kWidth * kHeight) / 4;
          } else {
            asU32[0] = value;
            const v = asF32[0];
            return v === -3 || v === 3;
          }
        }
      },
      {
        predicatePrinter: [
        {
          leftHeader: 'fb exp ==',
          getValueForCell: (idx) => {
            const x = idx % kWidth;
            const y = Math.floor(idx / kWidth);
            if (((x | y) & 0x1) === 0) {
              return kWidth * kHeight;
            } else {
              return useStorageBuffers ? 'any' : '+/- 3';
            }
          }
        }]

      }
    );
  };

  drawFullScreen(t, code, useStorageBuffers, dataChecker, fbChecker);
});