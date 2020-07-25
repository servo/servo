/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
vertexState validation tests.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';

import { ValidationTest } from './validation_test.js';

const MAX_VERTEX_ATTRIBUTES = 16;
const MAX_VERTEX_BUFFER_END = 2048;
const MAX_VERTEX_BUFFER_ARRAY_STRIDE = 2048;
const MAX_VERTEX_BUFFERS = 16;

const SIZEOF_FLOAT = Float32Array.BYTES_PER_ELEMENT;

const VERTEX_SHADER_CODE_WITH_NO_INPUT = `
  #version 450
  void main() {
    gl_Position = vec4(0.0);
  }
`;

const FRAGMENT_SHADER_CODE = `
  #version 450
  layout(location = 0) out vec4 fragColor;
  void main() {
    fragColor = vec4(0.0, 1.0, 0.0, 1.0);
  }
`;

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

class F extends ValidationTest {
  getDescriptor(vertexState, vertexShaderCode) {
    const descriptor = {
      vertexStage: {
        module: this.makeShaderModule('vertex', { glsl: vertexShaderCode }),
        entryPoint: 'main',
      },

      fragmentStage: {
        module: this.makeShaderModule('fragment', { glsl: FRAGMENT_SHADER_CODE }),
        entryPoint: 'main',
      },

      layout: this.device.createPipelineLayout({ bindGroupLayouts: [] }),
      primitiveTopology: 'triangle-list',
      colorStates: [{ format: 'rgba8unorm' }],
      vertexState,
    };

    return descriptor;
  }
}

export const g = makeTestGroup(F);

g.test('an_empty_vertex_input_is_valid').fn(t => {
  const vertexState = {};
  const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
  t.device.createRenderPipeline(descriptor);
});

g.test('a_null_buffer_is_valid').fn(t => {
  {
    // One null buffer is OK
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 0,
          attributes: [],
        },
      ],
    };

    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    //  One null buffer followed by a buffer is OK
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 0,
          attributes: [],
        },

        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 0,
            },
          ],
        },
      ],
    };

    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    //  One null buffer sitting between buffers is OK
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 0,
            },
          ],
        },

        {
          arrayStride: 0,
          attributes: [],
        },

        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 1,
            },
          ],
        },
      ],
    };

    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});

g.test('pipeline_vertex_buffers_are_backed_by_attributes_in_vertex_input').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 2 * SIZEOF_FLOAT,
        attributes: [
          {
            format: 'float',
            offset: 0,
            shaderLocation: 0,
          },

          {
            format: 'float',
            offset: 0,
            shaderLocation: 1,
          },
        ],
      },
    ],
  };

  {
    // Control case: pipeline with one input per attribute
    const code = `
      #version 450
      layout(location = 0) in vec4 a;
      layout(location = 1) in vec4 b;
      void main() {
          gl_Position = vec4(0.0);
      }
    `;
    const descriptor = t.getDescriptor(vertexState, code);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Check it is valid for the pipeline to use a subset of the VertexState
    const code = `
      #version 450
      layout(location = 0) in vec4 a;
      void main() {
          gl_Position = vec4(0.0);
      }
    `;
    const descriptor = t.getDescriptor(vertexState, code);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Check for an error when the pipeline uses an attribute not in the vertex input
    const code = `
      #version 450
      layout(location = 2) in vec4 a;
      void main() {
          gl_Position = vec4(0.0);
      }
    `;
    const descriptor = t.getDescriptor(vertexState, code);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('an_arrayStride_of_0_is_valid').fn(t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [
          {
            format: 'float',
            offset: 0,
            shaderLocation: 0,
          },
        ],
      },
    ],
  };

  {
    // Works ok without attributes
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Works ok with attributes at a large-ish offset
    vertexState.vertexBuffers[0].attributes[0].offset = 128;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});

g.test('offset_should_be_within_vertex_buffer_arrayStride_if_arrayStride_is_not_zero').fn(
  async t => {
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 2 * SIZEOF_FLOAT,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 0,
            },

            {
              format: 'float',
              offset: SIZEOF_FLOAT,
              shaderLocation: 1,
            },
          ],
        },
      ],
    };

    {
      // Control case, setting correct arrayStride and offset
      const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.device.createRenderPipeline(descriptor);
    }
    {
      // Test vertex attribute offset exceed vertex buffer arrayStride range
      const badVertexState = clone(vertexState);
      badVertexState.vertexBuffers[0].attributes[1].format = 'float2';
      const descriptor = t.getDescriptor(badVertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      });
    }
    {
      // Test vertex attribute offset exceed vertex buffer arrayStride range
      const badVertexState = clone(vertexState);
      badVertexState.vertexBuffers[0].arrayStride = SIZEOF_FLOAT;
      const descriptor = t.getDescriptor(badVertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      });
    }
    {
      // It's OK if arrayStride is zero
      const goodVertexState = clone(vertexState);
      goodVertexState.vertexBuffers[0].arrayStride = 0;
      const descriptor = t.getDescriptor(goodVertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.device.createRenderPipeline(descriptor);
    }
  }
);

// TODO: This should be made into an operation test.
g.test('check_two_attributes_overlapping').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 2 * SIZEOF_FLOAT,
        attributes: [
          {
            format: 'float',
            offset: 0,
            shaderLocation: 0,
          },

          {
            format: 'float',
            offset: SIZEOF_FLOAT,
            shaderLocation: 1,
          },
        ],
      },
    ],
  };

  {
    // Control case, setting correct arrayStride and offset
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test two attributes overlapping
    const overlappingVertexState = clone(vertexState);
    overlappingVertexState.vertexBuffers[0].attributes[0].format = 'int2';
    const descriptor = t.getDescriptor(overlappingVertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});

g.test('check_out_of_bounds_condition_on_total_number_of_vertex_buffers').fn(async t => {
  const vertexBuffers = [];

  for (let i = 0; i < MAX_VERTEX_BUFFERS; i++) {
    vertexBuffers.push({
      arrayStride: 0,
      attributes: [
        {
          format: 'float',
          offset: 0,
          shaderLocation: i,
        },
      ],
    });
  }
  {
    // Control case, setting max vertex buffer number
    const vertexState = { vertexBuffers };
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex buffer number exceed the limit
    const vertexState = {
      vertexBuffers: [
        ...vertexBuffers,
        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: MAX_VERTEX_BUFFERS,
            },
          ],
        },
      ],
    };

    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_out_of_bounds_on_number_of_vertex_attributes_on_a_single_vertex_buffer').fn(
  async t => {
    const vertexAttributes = [];

    for (let i = 0; i < MAX_VERTEX_ATTRIBUTES; i++) {
      vertexAttributes.push({
        format: 'float',
        offset: 0,
        shaderLocation: i,
      });
    }
    {
      // Control case, setting max vertex buffer number
      const vertexState = {
        vertexBuffers: [
          {
            arrayStride: 0,
            attributes: vertexAttributes,
          },
        ],
      };

      const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.device.createRenderPipeline(descriptor);
    }
    {
      // Test vertex attribute number exceed the limit
      const vertexState = {
        vertexBuffers: [
          {
            arrayStride: 0,
            attributes: [
              ...vertexAttributes,
              {
                format: 'float',
                offset: 0,
                shaderLocation: MAX_VERTEX_ATTRIBUTES,
              },
            ],
          },
        ],
      };

      const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      });
    }
  }
);

g.test('check_out_of_bounds_on_number_of_vertex_attributes_across_vertex_buffers').fn(async t => {
  const vertexBuffers = [];
  for (let i = 0; i < MAX_VERTEX_ATTRIBUTES; i++) {
    vertexBuffers.push({
      arrayStride: 0,
      attributes: [{ format: 'float', offset: 0, shaderLocation: i }],
    });
  }

  {
    // Control case, setting max vertex buffer number
    const vertexState = { vertexBuffers };
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex attribute number exceed the limit
    vertexBuffers[MAX_VERTEX_ATTRIBUTES - 1].attributes.push({
      format: 'float',
      offset: 0,
      shaderLocation: MAX_VERTEX_ATTRIBUTES,
    });

    const vertexState = { vertexBuffers };
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_out_of_bounds_condition_on_input_strides').fn(async t => {
  const vertexState = {
    vertexBuffers: [{ arrayStride: MAX_VERTEX_BUFFER_ARRAY_STRIDE, attributes: [] }],
  };

  {
    // Control case, setting max input arrayStride
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test input arrayStride OOB
    vertexState.vertexBuffers[0].arrayStride = MAX_VERTEX_BUFFER_ARRAY_STRIDE + 4;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_multiple_of_4_bytes_constraint_on_input_arrayStride').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 4,
        attributes: [{ format: 'uchar2', offset: 0, shaderLocation: 0 }],
      },
    ],
  };

  {
    // Control case, setting input arrayStride 4 bytes
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test input arrayStride not multiple of 4 bytes
    vertexState.vertexBuffers[0].arrayStride = 2;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('identical_duplicate_attributes_are_invalid').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [{ format: 'float', offset: 0, shaderLocation: 0 }],
      },
    ],
  };

  {
    // Control case, setting attribute 0
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Oh no, attribute 0 is set twice
    vertexState.vertexBuffers[0].attributes.push({
      format: 'float',
      offset: 0,
      shaderLocation: 0,
    });

    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('we_cannot_set_same_shader_location').fn(async t => {
  {
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 0,
          attributes: [
            { format: 'float', offset: 0, shaderLocation: 0 },
            { format: 'float', offset: SIZEOF_FLOAT, shaderLocation: 1 },
          ],
        },
      ],
    };

    {
      // Control case, setting different shader locations in two attributes
      const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.device.createRenderPipeline(descriptor);
    }
    {
      // Test same shader location in two attributes in the same buffer
      vertexState.vertexBuffers[0].attributes[1].shaderLocation = 0;
      const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      });
    }
  }
  {
    const vertexState = {
      vertexBuffers: [
        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 0,
            },
          ],
        },

        {
          arrayStride: 0,
          attributes: [
            {
              format: 'float',
              offset: 0,
              shaderLocation: 0,
            },
          ],
        },
      ],
    };

    // Test same shader location in two attributes in different buffers
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_out_of_bounds_condition_on_attribute_shader_location').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [{ format: 'float', offset: 0, shaderLocation: MAX_VERTEX_ATTRIBUTES - 1 }],
      },
    ],
  };

  {
    // Control case, setting last attribute shader location
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test attribute location OOB
    vertexState.vertexBuffers[0].attributes[0].shaderLocation = MAX_VERTEX_ATTRIBUTES;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_attribute_offset_out_of_bounds').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [
          {
            format: 'float2',
            offset: MAX_VERTEX_BUFFER_END - 2 * SIZEOF_FLOAT,
            shaderLocation: 0,
          },
        ],
      },
    ],
  };

  {
    // Control case, setting max attribute offset to MAX_VERTEX_BUFFER_END - 8
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Control case, setting attribute offset to 8
    vertexState.vertexBuffers[0].attributes[0].offset = 8;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test attribute offset out of bounds
    vertexState.vertexBuffers[0].attributes[0].offset = MAX_VERTEX_BUFFER_END - 4;
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_multiple_of_4_bytes_constraint_on_offset').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [{ format: 'float', offset: SIZEOF_FLOAT, shaderLocation: 0 }],
      },
    ],
  };

  {
    // Control case, setting offset 4 bytes
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test offset of 2 bytes with uchar2 format
    vertexState.vertexBuffers[0].attributes[0].offset = 2;
    vertexState.vertexBuffers[0].attributes[0].format = 'uchar2';
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
  {
    // Test offset of 2 bytes with float format
    vertexState.vertexBuffers[0].attributes[0].offset = 2;
    vertexState.vertexBuffers[0].attributes[0].format = 'float';
    const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});

g.test('check_attribute_offset_overflow').fn(async t => {
  const vertexState = {
    vertexBuffers: [
      {
        arrayStride: 0,
        attributes: [{ format: 'float', offset: Number.MAX_SAFE_INTEGER, shaderLocation: 0 }],
      },
    ],
  };

  const descriptor = t.getDescriptor(vertexState, VERTEX_SHADER_CODE_WITH_NO_INPUT);

  t.expectValidationError(() => {
    t.device.createRenderPipeline(descriptor);
  });
});
