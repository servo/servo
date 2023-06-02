/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This test dedicatedly tests validation of GPUVertexState of createRenderPipeline.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kMaxVertexAttributes,
  kMaxVertexBufferArrayStride,
  kMaxVertexBuffers,
  kVertexFormats,
  kVertexFormatInfo,
} from '../../../capability_info.js';
import { ValidationTest } from '../validation_test.js';

const VERTEX_SHADER_CODE_WITH_NO_INPUT = `
  @vertex fn main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
  }
`;

function addTestAttributes(
  attributes,
  {
    testAttribute,
    testAttributeAtStart = true,
    extraAttributeCount = 0,
    extraAttributeSkippedLocations = [],
  }
) {
  // Add a bunch of dummy attributes each with a different location such that none of the locations
  // are in extraAttributeSkippedLocations
  let currentLocation = 0;
  let extraAttribsAdded = 0;
  while (extraAttribsAdded !== extraAttributeCount) {
    if (extraAttributeSkippedLocations.includes(currentLocation)) {
      currentLocation++;
      continue;
    }

    attributes.push({ format: 'float32', shaderLocation: currentLocation, offset: 0 });
    currentLocation++;
    extraAttribsAdded++;
  }

  // Add the test attribute at the start or the end of the attributes.
  if (testAttribute) {
    if (testAttributeAtStart) {
      attributes.unshift(testAttribute);
    } else {
      attributes.push(testAttribute);
    }
  }
}

class F extends ValidationTest {
  getDescriptor(buffers, vertexShaderCode) {
    const descriptor = {
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({ code: vertexShaderCode }),
        entryPoint: 'main',
        buffers,
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
      primitive: { topology: 'triangle-list' },
    };
    return descriptor;
  }

  testVertexState(success, buffers, vertexShader = VERTEX_SHADER_CODE_WITH_NO_INPUT) {
    const vsModule = this.device.createShaderModule({ code: vertexShader });
    const fsModule = this.device.createShaderModule({
      code: `
        @fragment fn main() -> @location(0) vec4<f32> {
          return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }`,
    });

    this.expectValidationError(() => {
      this.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: vsModule,
          entryPoint: 'main',
          buffers,
        },
        fragment: {
          module: fsModule,
          entryPoint: 'main',
          targets: [{ format: 'rgba8unorm' }],
        },
        primitive: { topology: 'triangle-list' },
      });
    }, !success);
  }

  generateTestVertexShader(inputs) {
    let interfaces = '';
    let body = '';

    let count = 0;
    for (const input of inputs) {
      interfaces += `@location(${input.location}) input${count} : ${input.type},\n`;
      body += `var i${count} : ${input.type} = input.input${count};\n`;
      count++;
    }

    return `
      struct Inputs {
        ${interfaces}
      };
      @vertex fn main(input : Inputs) -> @builtin(position) vec4<f32> {
        ${body}
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
      }
    `;
  }
}

export const g = makeTestGroup(F);

g.test('max_vertex_buffer_limit')
  .desc(
    `Test that only up to <maxVertexBuffers> vertex buffers are allowed.
   - Tests with 0, 1, limits, limits + 1 vertex buffers.
   - Tests with the last buffer having an attribute or not.
  This also happens to test that vertex buffers with no attributes are allowed and that a vertex state with no buffers is allowed.`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('count', [0, 1, kMaxVertexBuffers, kMaxVertexBuffers + 1])
      .combine('lastEmpty', [false, true])
  )
  .fn(t => {
    const { count, lastEmpty } = t.params;

    const vertexBuffers = [];
    for (let i = 0; i < count; i++) {
      if (lastEmpty || i !== count - 1) {
        vertexBuffers.push({ attributes: [], arrayStride: 0 });
      } else {
        vertexBuffers.push({
          attributes: [{ format: 'float32', offset: 0, shaderLocation: 0 }],
          arrayStride: 0,
        });
      }
    }

    const success = count <= kMaxVertexBuffers;
    t.testVertexState(success, vertexBuffers);
  });

g.test('max_vertex_attribute_limit')
  .desc(
    `Test that only up to <maxVertexAttributes> vertex attributes are allowed.
   - Tests with 0, 1, limit, limits + 1 vertex attribute.
   - Tests with 0, 1, 4 attributes per buffer (with remaining attributes in the last buffer).`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('attribCount', [0, 1, kMaxVertexAttributes, kMaxVertexAttributes + 1])
      .combine('attribsPerBuffer', [0, 1, 4])
  )
  .fn(t => {
    const { attribCount, attribsPerBuffer } = t.params;

    const vertexBuffers = [];

    let attribsAdded = 0;
    while (attribsAdded !== attribCount) {
      // Choose how many attributes to add for this buffer. The last buffer gets all remaining attributes.
      let targetCount = Math.min(attribCount, attribsAdded + attribsPerBuffer);
      if (vertexBuffers.length === kMaxVertexBuffers - 1) {
        targetCount = attribCount;
      }

      const attributes = [];
      while (attribsAdded !== targetCount) {
        attributes.push({ format: 'float32', offset: 0, shaderLocation: attribsAdded });
        attribsAdded++;
      }

      vertexBuffers.push({ arrayStride: 0, attributes });
    }

    const success = attribCount <= kMaxVertexAttributes;
    t.testVertexState(success, vertexBuffers);
  });

g.test('max_vertex_buffer_array_stride_limit')
  .desc(
    `Test that the vertex buffer arrayStride must be at most <maxVertexBufferArrayStride>.
   - Test for various vertex buffer indices
   - Test for array strides 0, 4, 256, limit - 4, limit, limit + 4`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('arrayStride', [
        0,
        4,
        256,
        kMaxVertexBufferArrayStride - 4,
        kMaxVertexBufferArrayStride,
        kMaxVertexBufferArrayStride + 4,
      ])
  )
  .fn(t => {
    const { vertexBufferIndex, arrayStride } = t.params;

    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride, attributes: [] };

    const success = arrayStride <= kMaxVertexBufferArrayStride;
    t.testVertexState(success, vertexBuffers);
  });

g.test('vertex_buffer_array_stride_limit_alignment')
  .desc(
    `Test that the vertex buffer arrayStride must be a multiple of 4 (including 0).
   - Test for various vertex buffer indices
   - Test for array strides 0, 1, 2, 4, limit - 4, limit - 2, limit`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('arrayStride', [
        0,
        1,
        2,
        4,
        kMaxVertexBufferArrayStride - 4,
        kMaxVertexBufferArrayStride - 2,
        kMaxVertexBufferArrayStride,
      ])
  )
  .fn(t => {
    const { vertexBufferIndex, arrayStride } = t.params;

    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride, attributes: [] };

    const success = arrayStride % 4 === 0;
    t.testVertexState(success, vertexBuffers);
  });

g.test('vertex_attribute_shaderLocation_limit')
  .desc(
    `Test shaderLocation must be less than maxVertexAttributes.
   - Test for various vertex buffer indices
   - Test for various amounts of attributes in that vertex buffer
   - Test for shaderLocation 0, 1, limit - 1, limit`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('extraAttributeCount', [0, 1, kMaxVertexAttributes - 1])
      .combine('testAttributeAtStart', [false, true])
      .combine('testShaderLocation', [0, 1, kMaxVertexAttributes - 1, kMaxVertexAttributes])
  )
  .fn(t => {
    const {
      vertexBufferIndex,
      extraAttributeCount,
      testShaderLocation,
      testAttributeAtStart,
    } = t.params;

    const attributes = [];
    addTestAttributes(attributes, {
      testAttribute: { format: 'float32', offset: 0, shaderLocation: testShaderLocation },
      testAttributeAtStart,
      extraAttributeCount,
      extraAttributeSkippedLocations: [testShaderLocation],
    });

    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride: 256, attributes };

    const success = testShaderLocation < kMaxVertexAttributes;
    t.testVertexState(success, vertexBuffers);
  });

g.test('vertex_attribute_shaderLocation_unique')
  .desc(
    `Test that shaderLocation must be unique in the vertex state.
   - Test for various pairs of buffers that contain the potentially conflicting attributes
   - Test for the potentially conflicting attributes in various places in the buffers (with dummy attributes)
   - Test for various shaderLocations that conflict or not`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('vertexBufferIndexA', [0, 1, kMaxVertexBuffers - 1])
      .combine('vertexBufferIndexB', [0, 1, kMaxVertexBuffers - 1])
      .combine('testAttributeAtStartA', [false, true])
      .combine('testAttributeAtStartB', [false, true])
      .combine('shaderLocationA', [0, 1, 7, kMaxVertexAttributes - 1])
      .combine('shaderLocationB', [0, 1, 7, kMaxVertexAttributes - 1])
      .combine('extraAttributeCount', [0, 4])
  )
  .fn(t => {
    const {
      vertexBufferIndexA,
      vertexBufferIndexB,
      testAttributeAtStartA,
      testAttributeAtStartB,
      shaderLocationA,
      shaderLocationB,
      extraAttributeCount,
    } = t.params;

    // Depending on the params, the vertexBuffer for A and B can be the same or different. To support
    // both cases without code changes we treat `vertexBufferAttributes` as a map from indices to
    // vertex buffer descriptors, with A and B potentially reusing the same JS object if they have the
    // same index.
    const vertexBufferAttributes = [];
    vertexBufferAttributes[vertexBufferIndexA] = [];
    vertexBufferAttributes[vertexBufferIndexB] = [];

    // Add the dummy attributes for attribute A
    const attributesA = vertexBufferAttributes[vertexBufferIndexA];
    addTestAttributes(attributesA, {
      testAttribute: { format: 'float32', offset: 0, shaderLocation: shaderLocationA },
      testAttributeAtStart: testAttributeAtStartA,
      extraAttributeCount,
      extraAttributeSkippedLocations: [shaderLocationA, shaderLocationB],
    });

    // Add attribute B. Not that attributesB can be the same object as attributesA so they end
    // up in the same vertex buffer.
    const attributesB = vertexBufferAttributes[vertexBufferIndexB];
    addTestAttributes(attributesB, {
      testAttribute: { format: 'float32', offset: 0, shaderLocation: shaderLocationB },
      testAttributeAtStart: testAttributeAtStartB,
    });

    // Use the attributes to make the list of vertex buffers. Note that we might be setting the same vertex
    // buffer twice, but that only happens when it is the only vertex buffer.
    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndexA] = { arrayStride: 256, attributes: attributesA };
    vertexBuffers[vertexBufferIndexB] = { arrayStride: 256, attributes: attributesB };

    // Note that an empty vertex shader will be used so errors only happens because of the conflict
    // in the vertex state.
    const success = shaderLocationA !== shaderLocationB;
    t.testVertexState(success, vertexBuffers);
  });

g.test('vertex_shader_input_location_limit')
  .desc(
    `Test that vertex shader's input's location decoration must be less than maxVertexAttributes.
   - Test for shaderLocation 0, 1, limit - 1, limit, MAX_I32 (the WGSL spec requires a non-negative i32)`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('testLocation', [0, 1, kMaxVertexAttributes - 1, kMaxVertexAttributes, 2 ** 31 - 1])
  )
  .fn(t => {
    const { testLocation } = t.params;

    const shader = t.generateTestVertexShader([
      {
        type: 'vec4<f32>',
        location: testLocation,
      },
    ]);

    const vertexBuffers = [
      {
        arrayStride: 512,
        attributes: [
          {
            format: 'float32',
            offset: 0,
            shaderLocation: testLocation,
          },
        ],
      },
    ];

    const success = testLocation < kMaxVertexAttributes;
    t.testVertexState(success, vertexBuffers, shader);
  });

g.test('vertex_shader_input_location_in_vertex_state')
  .desc(
    `Test that a vertex shader defined in the shader must have a corresponding attribute in the vertex state.
       - Test for various input locations.
       - Test for the attribute in various places in the list of vertex buffer and various places inside the vertex buffer descriptor`
  )
  .paramsSubcasesOnly(u =>
    u //
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('extraAttributeCount', [0, 1, kMaxVertexAttributes - 1])
      .combine('testAttributeAtStart', [false, true])
      .combine('testShaderLocation', [0, 1, 4, 7, kMaxVertexAttributes - 1])
  )
  .fn(t => {
    const {
      vertexBufferIndex,
      extraAttributeCount,
      testAttributeAtStart,
      testShaderLocation,
    } = t.params;
    // We have a shader using `testShaderLocation`.
    const shader = t.generateTestVertexShader([
      {
        type: 'vec4<f32>',
        location: testShaderLocation,
      },
    ]);

    const attributes = [];
    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride: 256, attributes };

    // Fill attributes with a bunch of attributes for other locations.
    // Using that vertex state is invalid because the vertex state doesn't contain the test location
    addTestAttributes(attributes, {
      extraAttributeCount,
      extraAttributeSkippedLocations: [testShaderLocation],
    });
    t.testVertexState(false, vertexBuffers, shader);

    // Add an attribute for the test location and try again.
    addTestAttributes(attributes, {
      testAttribute: { format: 'float32', shaderLocation: testShaderLocation, offset: 0 },
      testAttributeAtStart,
    });
    t.testVertexState(true, vertexBuffers, shader);
  });

g.test('vertex_shader_type_matches_attribute_format')
  .desc(
    `
    Test that the vertex shader declaration must have a type compatible with the vertex format.
     - Test for all formats.
     - Test for all combinations of u/i/f32 with and without vectors.`
  )
  .params(u =>
    u
      .combine('format', kVertexFormats)
      .beginSubcases()
      .combine('shaderBaseType', ['u32', 'i32', 'f32'])
      .expand('shaderType', p => [
        p.shaderBaseType,
        `vec2<${p.shaderBaseType}>`,
        `vec3<${p.shaderBaseType}>`,
        `vec4<${p.shaderBaseType}>`,
      ])
  )
  .fn(t => {
    const { format, shaderBaseType, shaderType } = t.params;
    const shader = t.generateTestVertexShader([
      {
        type: shaderType,
        location: 0,
      },
    ]);

    const requiredBaseType = {
      sint: 'i32',
      uint: 'u32',
      snorm: 'f32',
      unorm: 'f32',
      float: 'f32',
    }[kVertexFormatInfo[format].type];

    const success = requiredBaseType === shaderBaseType;
    t.testVertexState(
      success,
      [
        {
          arrayStride: 0,
          attributes: [{ offset: 0, shaderLocation: 0, format }],
        },
      ],

      shader
    );
  });

g.test('vertex_attribute_offset_alignment')
  .desc(
    `
    Test that vertex attribute offsets must be aligned to the format's component byte size.
    - Test for all formats.
    - Test for various arrayStrides and offsets within that stride
    - Test for various vertex buffer indices
    - Test for various amounts of attributes in that vertex buffer`
  )
  .params(u =>
    u
      .combine('format', kVertexFormats)
      .combine('arrayStride', [256, kMaxVertexBufferArrayStride])
      .expand('offset', p => {
        const { bytesPerComponent, componentCount } = kVertexFormatInfo[p.format];
        const formatSize = bytesPerComponent * componentCount;

        return new Set([
          0,
          Math.floor(formatSize / 2),
          formatSize,
          2,
          4,
          p.arrayStride - formatSize,
          p.arrayStride - formatSize - Math.floor(formatSize / 2),
          p.arrayStride - formatSize - 4,
          p.arrayStride - formatSize - 2,
        ]);
      })
      .beginSubcases()
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('extraAttributeCount', [0, 1, kMaxVertexAttributes - 1])
      .combine('testAttributeAtStart', [false, true])
  )
  .fn(t => {
    const {
      format,
      arrayStride,
      offset,
      vertexBufferIndex,
      extraAttributeCount,
      testAttributeAtStart,
    } = t.params;

    const attributes = [];
    addTestAttributes(attributes, {
      testAttribute: { format, offset, shaderLocation: 0 },
      testAttributeAtStart,
      extraAttributeCount,
      extraAttributeSkippedLocations: [0],
    });

    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride, attributes };

    const formatInfo = kVertexFormatInfo[format];
    const formatSize = formatInfo.bytesPerComponent * formatInfo.componentCount;
    const success = offset % Math.min(4, formatSize) === 0;

    t.testVertexState(success, vertexBuffers);
  });

g.test('vertex_attribute_contained_in_stride')
  .desc(
    `
    Test that vertex attribute [offset, offset + formatSize) must be contained in the arrayStride if arrayStride is not 0:
    - Test for all formats.
    - Test for various arrayStrides and offsets within that stride
    - Test for various vertex buffer indices
    - Test for various amounts of attributes in that vertex buffer`
  )
  .params(u =>
    u
      .combine('format', kVertexFormats)
      .beginSubcases()
      .combine('arrayStride', [
        0,
        256,
        kMaxVertexBufferArrayStride - 4,
        kMaxVertexBufferArrayStride,
      ])
      .expand('offset', function* (p) {
        // Compute a bunch of test offsets to test.
        const { bytesPerComponent, componentCount } = kVertexFormatInfo[p.format];
        const formatSize = bytesPerComponent * componentCount;
        yield 0;
        yield 4;

        // arrayStride = 0 is a special case because for the offset validation it acts the same
        // as arrayStride = kMaxVertexBufferArrayStride. We special case here so as to avoid adding
        // negative offsets that would cause an IDL exception to be thrown instead of a validation
        // error.
        const stride = p.arrayStride !== 0 ? p.arrayStride : kMaxVertexBufferArrayStride;
        yield stride - formatSize;
        yield stride - formatSize + 4;

        // Avoid adding duplicate cases when formatSize == 4 (it is already tested above)
        if (formatSize !== 4) {
          yield formatSize;
          yield stride;
        }
      })
      .combine('vertexBufferIndex', [0, 1, kMaxVertexBuffers - 1])
      .combine('extraAttributeCount', [0, 1, kMaxVertexAttributes - 1])
      .combine('testAttributeAtStart', [false, true])
  )
  .fn(t => {
    const {
      format,
      arrayStride,
      offset,
      vertexBufferIndex,
      extraAttributeCount,
      testAttributeAtStart,
    } = t.params;

    const attributes = [];
    addTestAttributes(attributes, {
      testAttribute: { format, offset, shaderLocation: 0 },
      testAttributeAtStart,
      extraAttributeCount,
      extraAttributeSkippedLocations: [0],
    });

    const vertexBuffers = [];
    vertexBuffers[vertexBufferIndex] = { arrayStride, attributes };

    const formatInfo = kVertexFormatInfo[format];
    const formatSize = formatInfo.bytesPerComponent * formatInfo.componentCount;
    const limit = arrayStride === 0 ? kMaxVertexBufferArrayStride : arrayStride;

    const success = offset + formatSize <= limit;
    t.testVertexState(success, vertexBuffers);
  });

g.test('many_attributes_overlapping')
  .desc(`Test that it is valid to have many vertex attributes overlap`)
  .fn(t => {
    // Create many attributes, each of them intersects with at least 3 others.
    const attributes = [];
    const formats = ['float32x4', 'uint32x4', 'sint32x4'];
    for (let i = 0; i < kMaxVertexAttributes; i++) {
      attributes.push({ format: formats[i % 3], offset: i * 4, shaderLocation: i });
    }

    t.testVertexState(true, [{ arrayStride: 0, attributes }]);
  });
