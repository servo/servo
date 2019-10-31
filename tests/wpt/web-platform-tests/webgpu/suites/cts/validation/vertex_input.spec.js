/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
vertexInput validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';
const MAX_VERTEX_ATTRIBUTES = 16;
const MAX_VERTEX_BUFFER_END = 2048;
const MAX_VERTEX_BUFFER_STRIDE = 2048;
const MAX_VERTEX_BUFFERS = 16;
const VERTEX_SHADER_CODE_WITH_NO_INPUT = `
  #version 450
  void main() {
    gl_Position = vec4(0.0);
  }
`;

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

class F extends ValidationTest {
  async init() {
    await Promise.all([super.init(), this.initGLSL()]);
  }

  getDescriptor(vertexInput, vertexShaderCode) {
    const descriptor = {
      vertexStage: this.getVertexStage(vertexShaderCode),
      fragmentStage: this.getFragmentStage(),
      layout: this.getPipelineLayout(),
      primitiveTopology: 'triangle-list',
      colorStates: [{
        format: 'rgba8unorm'
      }],
      vertexInput
    };
    return descriptor;
  }

  getVertexStage(code) {
    return {
      module: this.makeShaderModule('vertex', code),
      entryPoint: 'main'
    };
  }

  getFragmentStage() {
    const code = `
      #version 450
      layout(location = 0) out vec4 fragColor;
      void main() {
        fragColor = vec4(0.0, 1.0, 0.0, 1.0);
      }
    `;
    return {
      module: this.makeShaderModule('fragment', code),
      entryPoint: 'main'
    };
  }

  getPipelineLayout() {
    return this.device.createPipelineLayout({
      bindGroupLayouts: []
    });
  }

}

export const g = new TestGroup(F);
g.test('an empty vertex input is valid', t => {
  const vertexInput = {};
  const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
  t.device.createRenderPipeline(descriptor);
});
g.test('a null buffer is valid', t => {
  {
    // One null buffer is OK
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: []
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    //  One null buffer followed by a buffer is OK
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: []
      }, {
        stride: 0,
        attributeSet: [{
          shaderLocation: 0,
          format: 'float'
        }]
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    //  One null buffer sitting between buffers is OK
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: [{
          shaderLocation: 0,
          format: 'float'
        }]
      }, {
        stride: 0,
        attributeSet: []
      }, {
        stride: 0,
        attributeSet: [{
          shaderLocation: 1,
          format: 'float'
        }]
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});
g.test('pipeline vertex buffers are backed by attributes in vertex input', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 2 * Float32Array.BYTES_PER_ELEMENT,
      attributeSet: [{
        shaderLocation: 0,
        format: 'float'
      }, {
        shaderLocation: 1,
        format: 'float'
      }]
    }]
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
    const descriptor = t.getDescriptor(vertexInput, code);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Check it is valid for the pipeline to use a subset of the VertexInput
    const code = `
      #version 450
      layout(location = 0) in vec4 a;
      void main() {
          gl_Position = vec4(0.0);
      }
    `;
    const descriptor = t.getDescriptor(vertexInput, code);
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
    const descriptor = t.getDescriptor(vertexInput, code);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('a stride of 0 is valid', t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        shaderLocation: 0,
        format: 'float'
      }]
    }]
  };
  {
    // Works ok without attributes
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Works ok with attributes at a large-ish offset
    vertexInput.vertexBuffers[0].attributeSet[0].offset = 128;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});
g.test('offset should be within vertex buffer stride if stride is not zero', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 2 * Float32Array.BYTES_PER_ELEMENT,
      attributeSet: [{
        shaderLocation: 0,
        format: 'float'
      }, {
        offset: Float32Array.BYTES_PER_ELEMENT,
        shaderLocation: 1,
        format: 'float'
      }]
    }]
  };
  {
    // Control case, setting correct stride and offset
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex attribute offset exceed vertex buffer stride range
    const badVertexInput = clone(vertexInput);
    badVertexInput.vertexBuffers[0].attributeSet[1].format = 'float2';
    const descriptor = t.getDescriptor(badVertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
  {
    // Test vertex attribute offset exceed vertex buffer stride range
    const badVertexInput = clone(vertexInput);
    badVertexInput.vertexBuffers[0].stride = Float32Array.BYTES_PER_ELEMENT;
    const descriptor = t.getDescriptor(badVertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
  {
    // It's OK if stride is zero
    const goodVertexInput = clone(vertexInput);
    goodVertexInput.vertexBuffers[0].stride = 0;
    const descriptor = t.getDescriptor(goodVertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
});
g.test('check two attributes overlapping', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 2 * Float32Array.BYTES_PER_ELEMENT,
      attributeSet: [{
        shaderLocation: 0,
        format: 'float'
      }, {
        offset: Float32Array.BYTES_PER_ELEMENT,
        shaderLocation: 1,
        format: 'float'
      }]
    }]
  };
  {
    // Control case, setting correct stride and offset
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test two attributes overlapping
    const badVertexInput = clone(vertexInput);
    badVertexInput.vertexBuffers[0].attributeSet[0].format = 'int2';
    const descriptor = t.getDescriptor(badVertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check out of bounds condition on total number of vertex buffers', async t => {
  const vertexBuffers = [];

  for (let i = 0; i < MAX_VERTEX_BUFFERS; i++) {
    vertexBuffers.push({
      stride: 0,
      attributeSet: [{
        shaderLocation: i,
        format: 'float'
      }]
    });
  }

  {
    // Control case, setting max vertex buffer number
    const vertexInput = {
      vertexBuffers
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex buffer number exceed the limit
    const vertexInput = {
      vertexBuffers: [...vertexBuffers, {
        stride: 0,
        attributeSet: [{
          shaderLocation: MAX_VERTEX_BUFFERS,
          format: 'float'
        }]
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check out of bounds on number of vertex attributes on a single vertex buffer', async t => {
  const vertexAttributes = [];

  for (let i = 0; i < MAX_VERTEX_ATTRIBUTES; i++) {
    vertexAttributes.push({
      shaderLocation: i,
      format: 'float'
    });
  }

  {
    // Control case, setting max vertex buffer number
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: vertexAttributes
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex attribute number exceed the limit
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: [...vertexAttributes, {
          shaderLocation: MAX_VERTEX_ATTRIBUTES,
          format: 'float'
        }]
      }]
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check out of bounds on number of vertex attributes across vertex buffers', async t => {
  const vertexBuffers = [];

  for (let i = 0; i < MAX_VERTEX_ATTRIBUTES; i++) {
    vertexBuffers.push({
      stride: 0,
      attributeSet: [{
        shaderLocation: i,
        format: 'float'
      }]
    });
  }

  {
    // Control case, setting max vertex buffer number
    const vertexInput = {
      vertexBuffers
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test vertex attribute number exceed the limit
    vertexBuffers[MAX_VERTEX_ATTRIBUTES - 1].attributeSet.push({
      shaderLocation: MAX_VERTEX_ATTRIBUTES,
      format: 'float'
    });
    const vertexInput = {
      vertexBuffers
    };
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check out of bounds condition on input strides', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: MAX_VERTEX_BUFFER_STRIDE,
      attributeSet: []
    }]
  };
  {
    // Control case, setting max input stride
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test input stride OOB
    vertexInput.vertexBuffers[0].stride = MAX_VERTEX_BUFFER_STRIDE + 4;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check multiple of 4 bytes constraint on input stride', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 4,
      attributeSet: [{
        shaderLocation: 0,
        format: 'uchar2'
      }]
    }]
  };
  {
    // Control case, setting input stride 4 bytes
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test input stride not multiple of 4 bytes
    vertexInput.vertexBuffers[0].stride = 2;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('identical duplicate attributes are invalid', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        shaderLocation: 0,
        format: 'float'
      }]
    }]
  };
  {
    // Control case, setting attribute 0
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Oh no, attribute 0 is set twice
    vertexInput.vertexBuffers[0].attributeSet.push({
      shaderLocation: 0,
      format: 'float'
    });
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('we cannot set same shader location', async t => {
  {
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: [{
          shaderLocation: 0,
          format: 'float'
        }, {
          offset: Float32Array.BYTES_PER_ELEMENT,
          shaderLocation: 1,
          format: 'float'
        }]
      }]
    };
    {
      // Control case, setting different shader locations in two attributes
      const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.device.createRenderPipeline(descriptor);
    }
    {
      // Test same shader location in two attributes in the same buffer
      vertexInput.vertexBuffers[0].attributeSet[1].shaderLocation = 0;
      const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
      t.expectValidationError(() => {
        t.device.createRenderPipeline(descriptor);
      });
    }
  }
  {
    const vertexInput = {
      vertexBuffers: [{
        stride: 0,
        attributeSet: [{
          shaderLocation: 0,
          format: 'float'
        }]
      }, {
        stride: 0,
        attributeSet: [{
          shaderLocation: 0,
          format: 'float'
        }]
      }]
    }; // Test same shader location in two attributes in different buffers

    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check out of bounds condition on attribute shader location', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        shaderLocation: MAX_VERTEX_ATTRIBUTES - 1,
        format: 'float'
      }]
    }]
  };
  {
    // Control case, setting last attribute shader location
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test attribute location OOB
    vertexInput.vertexBuffers[0].attributeSet[0].shaderLocation = MAX_VERTEX_ATTRIBUTES;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check attribute offset out of bounds', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        offset: MAX_VERTEX_BUFFER_END - 2 * Float32Array.BYTES_PER_ELEMENT,
        shaderLocation: 0,
        format: 'float2'
      }]
    }]
  };
  {
    // Control case, setting max attribute offset to MAX_VERTEX_BUFFER_END - 8
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Control case, setting attribute offset to 8
    vertexInput.vertexBuffers[0].attributeSet[0].offset = 8;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test attribute offset out of bounds
    vertexInput.vertexBuffers[0].attributeSet[0].offset = MAX_VERTEX_BUFFER_END - 4;
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check multiple of 4 bytes constraint on offset', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        offset: Float32Array.BYTES_PER_ELEMENT,
        shaderLocation: 0,
        format: 'float'
      }]
    }]
  };
  {
    // Control case, setting offset 4 bytes
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.device.createRenderPipeline(descriptor);
  }
  {
    // Test offset of 2 bytes with uchar2 format
    vertexInput.vertexBuffers[0].attributeSet[0].offset = 2;
    vertexInput.vertexBuffers[0].attributeSet[0].format = 'uchar2';
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
  {
    // Test offset of 2 bytes with float format
    vertexInput.vertexBuffers[0].attributeSet[0].offset = 2;
    vertexInput.vertexBuffers[0].attributeSet[0].format = 'float';
    const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
    t.expectValidationError(() => {
      t.device.createRenderPipeline(descriptor);
    });
  }
});
g.test('check attribute offset overflow', async t => {
  const vertexInput = {
    vertexBuffers: [{
      stride: 0,
      attributeSet: [{
        offset: Number.MAX_SAFE_INTEGER,
        shaderLocation: 0,
        format: 'float'
      }]
    }]
  };
  const descriptor = t.getDescriptor(vertexInput, VERTEX_SHADER_CODE_WITH_NO_INPUT);
  t.expectValidationError(() => {
    t.device.createRenderPipeline(descriptor);
  });
});
//# sourceMappingURL=vertex_input.spec.js.map