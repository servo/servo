/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ function _defineProperty(obj, key, value) {
  if (key in obj) {
    Object.defineProperty(obj, key, {
      value: value,
      enumerable: true,
      configurable: true,
      writable: true,
    });
  } else {
    obj[key] = value;
  }
  return obj;
}
export const description = `
Test vertex attributes behave correctly (no crash / data leak) when accessed out of bounds

Test coverage:

The following will be parameterized (all combinations tested):

1) Draw call indexed? (false / true)
  - Run the draw call using an index buffer

2) Draw call indirect? (false / true)
  - Run the draw call using an indirect buffer

3) Draw call parameter (vertexCount, firstVertex, indexCount, firstIndex, baseVertex, instanceCount,
  firstInstance)
  - The parameter which will go out of bounds. Filtered depending on if the draw call is indexed.

4) Attribute type (float, vec2, vec3, vec4)
  - The input attribute type in the vertex shader

5) Error scale (1, 4, 10^2, 10^4, 10^6)
  - Offset to add to the correct draw call parameter

6) Additional vertex buffers (0, +4)
  - Tests that no OOB occurs if more vertex buffers are used

The tests will also have another vertex buffer bound for an instanced attribute, to make sure
instanceCount / firstInstance are tested.

The tests will include multiple attributes per vertex buffer.

The vertex buffers will be filled by repeating a few chosen values until the end of the buffer.

The test will run a render pipeline which verifies the following:
1) All vertex attribute values occur in the buffer or are zero
2) All gl_VertexIndex values are within the index buffer or 0

TODO:

A suppression may be needed for d3d12 on tests that have non-zero baseVertex, since d3d12 counts
from 0 instead of from baseVertex (will fail check for gl_VertexIndex).

Vertex buffer contents could be randomized to prevent the case where a previous test creates
a similar buffer to ours and the OOB-read seems valid. This should be deterministic, which adds
more complexity that we may not need.`;
import { params, pbool, poptions } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

// Encapsulates a draw call (either indexed or non-indexed)
class DrawCall {
  // Add a float offset when binding vertex buffer

  // Draw

  // DrawIndexed

  // Both Draw and DrawIndexed

  constructor(device, vertexArrays, vertexCount, partialLastNumber, offsetVertexBuffer) {
    _defineProperty(this, 'device', void 0);
    _defineProperty(this, 'vertexBuffers', void 0);
    _defineProperty(this, 'indexBuffer', void 0);
    _defineProperty(this, 'offsetVertexBuffer', void 0);
    _defineProperty(this, 'vertexCount', void 0);
    _defineProperty(this, 'firstVertex', void 0);
    _defineProperty(this, 'indexCount', void 0);
    _defineProperty(this, 'firstIndex', void 0);
    _defineProperty(this, 'baseVertex', void 0);
    _defineProperty(this, 'instanceCount', void 0);
    _defineProperty(this, 'firstInstance', void 0);
    this.device = device;
    this.vertexBuffers = vertexArrays.map(v => this.generateVertexBuffer(v, partialLastNumber));

    const indexArray = new Uint16Array(vertexCount).fill(0).map((_, i) => i);
    this.indexBuffer = this.generateIndexBuffer(indexArray);

    // Default arguments (valid call)
    this.vertexCount = vertexCount;
    this.firstVertex = 0;
    this.indexCount = vertexCount;
    this.firstIndex = 0;
    this.baseVertex = 0;
    this.instanceCount = vertexCount;
    this.firstInstance = 0;

    this.offsetVertexBuffer = offsetVertexBuffer;
  }

  // Insert a draw call into |pass| with specified type
  insertInto(pass, indexed, indirect) {
    if (indexed) {
      if (indirect) {
        this.drawIndexedIndirect(pass);
      } else {
        this.drawIndexed(pass);
      }
    } else {
      if (indirect) {
        this.drawIndirect(pass);
      } else {
        this.draw(pass);
      }
    }
  }

  // Insert a draw call into |pass|
  draw(pass) {
    this.bindVertexBuffers(pass);
    pass.draw(this.vertexCount, this.instanceCount, this.firstVertex, this.firstInstance);
  }

  // Insert an indexed draw call into |pass|
  drawIndexed(pass) {
    this.bindVertexBuffers(pass);
    pass.setIndexBuffer(this.indexBuffer);
    pass.drawIndexed(
      this.indexCount,
      this.instanceCount,
      this.firstIndex,
      this.baseVertex,
      this.firstInstance
    );
  }

  // Insert an indirect draw call into |pass|
  drawIndirect(pass) {
    this.bindVertexBuffers(pass);
    pass.drawIndirect(this.generateIndirectBuffer(), 0);
  }

  // Insert an indexed indirect draw call into |pass|
  drawIndexedIndirect(pass) {
    this.bindVertexBuffers(pass);
    pass.setIndexBuffer(this.indexBuffer);
    pass.drawIndexedIndirect(this.generateIndexedIndirectBuffer(), 0);
  }

  // Bind all vertex buffers generated
  bindVertexBuffers(pass) {
    let currSlot = 0;
    for (let i = 0; i < this.vertexBuffers.length; i++) {
      pass.setVertexBuffer(currSlot++, this.vertexBuffers[i], this.offsetVertexBuffer ? 4 : 0);
    }
  }

  // Create a vertex buffer from |vertexArray|
  // If |partialLastNumber| is true, delete one byte off the end
  generateVertexBuffer(vertexArray, partialLastNumber) {
    let size = vertexArray.byteLength;
    if (partialLastNumber) {
      size -= 1;
    }
    const [vertexBuffer, vertexMapping] = this.device.createBufferMapped({
      size,
      usage: GPUBufferUsage.VERTEX,
    });

    if (!partialLastNumber) {
      new Float32Array(vertexMapping).set(vertexArray);
    } else {
      new Uint8Array(vertexMapping).set(new Uint8Array(vertexArray.buffer).slice(0, size));
    }
    vertexBuffer.unmap();
    return vertexBuffer;
  }

  // Create an index buffer from |indexArray|
  generateIndexBuffer(indexArray) {
    const [indexBuffer, indexMapping] = this.device.createBufferMapped({
      size: indexArray.byteLength,
      usage: GPUBufferUsage.INDEX,
    });

    new Uint16Array(indexMapping).set(indexArray);
    indexBuffer.unmap();
    return indexBuffer;
  }

  // Create an indirect buffer containing draw call values
  generateIndirectBuffer() {
    const indirectArray = new Int32Array([
      this.vertexCount,
      this.instanceCount,
      this.firstVertex,
      this.firstInstance,
    ]);

    const [indirectBuffer, indirectMapping] = this.device.createBufferMapped({
      size: indirectArray.byteLength,
      usage: GPUBufferUsage.INDIRECT,
    });

    new Int32Array(indirectMapping).set(indirectArray);
    indirectBuffer.unmap();
    return indirectBuffer;
  }

  // Create an indirect buffer containing indexed draw call values
  generateIndexedIndirectBuffer() {
    const indirectArray = new Int32Array([
      this.indexCount,
      this.instanceCount,
      this.firstVertex,
      this.baseVertex,
      this.firstInstance,
    ]);

    const [indirectBuffer, indirectMapping] = this.device.createBufferMapped({
      size: indirectArray.byteLength,
      usage: GPUBufferUsage.INDIRECT,
    });

    new Int32Array(indirectMapping).set(indirectArray);
    indirectBuffer.unmap();
    return indirectBuffer;
  }
}

// Parameterize different sized types

const typeInfoMap = {
  float: {
    format: 'float',
    size: 4,
    validationFunc: 'return valid(v);',
  },

  vec2: {
    format: 'float2',
    size: 8,
    validationFunc: 'return valid(v.x) && valid(v.y);',
  },

  vec3: {
    format: 'float3',
    size: 12,
    validationFunc: 'return valid(v.x) && valid(v.y) && valid(v.z);',
  },

  vec4: {
    format: 'float4',
    size: 16,
    validationFunc: `return valid(v.x) && valid(v.y) && valid(v.z) && valid(v.w) ||
                            v.x == 0 && v.y == 0 && v.z == 0 && (v.w == 0.0 || v.w == 1.0);`,
  },
};

g.test('vertexAccess')
  .params(
    params()
      .combine(pbool('indexed'))
      .combine(pbool('indirect'))
      .expand(p =>
        poptions(
          'drawCallTestParameter',
          p.indexed
            ? ['indexCount', 'instanceCount', 'firstIndex', 'baseVertex', 'firstInstance']
            : ['vertexCount', 'instanceCount', 'firstVertex', 'firstInstance']
        )
      )
      .combine(poptions('type', Object.keys(typeInfoMap)))
      .combine(poptions('additionalBuffers', [0, 4]))
      .combine(pbool('partialLastNumber'))
      .combine(pbool('offsetVertexBuffer'))
      .combine(poptions('errorScale', [1, 4, 10 ** 2, 10 ** 4, 10 ** 6]))
  )
  .fn(async t => {
    const p = t.params;
    const typeInfo = typeInfoMap[p.type];

    // Number of vertices to draw
    const numVertices = 3;
    // Each buffer will be bound to this many attributes (2 would mean 2 attributes per buffer)
    const attributesPerBuffer = 2;
    // Make an array big enough for the vertices, attributes, and size of each element
    const vertexArray = new Float32Array(numVertices * attributesPerBuffer * (typeInfo.size / 4));

    // Sufficiently unusual values to fill our buffer with to avoid collisions with other tests
    const arbitraryValues = [759, 329, 908];
    for (let i = 0; i < vertexArray.length; ++i) {
      vertexArray[i] = arbitraryValues[i % arbitraryValues.length];
    }
    // A valid value is 0 or one in the buffer
    const validValues = [0, ...arbitraryValues];

    // Instance step mode buffer, vertex step mode buffer
    const bufferContents = [vertexArray, vertexArray];
    // Additional buffers (vertex step mode)
    for (let i = 0; i < p.additionalBuffers; i++) {
      bufferContents.push(vertexArray);
    }

    // Mutable draw call
    const draw = new DrawCall(
      t.device,
      bufferContents,
      numVertices,
      p.partialLastNumber,
      p.offsetVertexBuffer
    );

    // Create attributes listing
    let layoutStr = '';
    const attributeNames = [];
    {
      let currAttribute = 0;
      for (let i = 0; i < bufferContents.length; i++) {
        for (let j = 0; j < attributesPerBuffer; j++) {
          layoutStr += `layout(location=${currAttribute}) in ${p.type} a_${currAttribute};\n`;
          attributeNames.push(`a_${currAttribute}`);
          currAttribute++;
        }
      }
    }

    // Vertex buffer descriptors
    const vertexBuffers = [];
    {
      let currAttribute = 0;
      for (let i = 0; i < bufferContents.length; i++) {
        vertexBuffers.push({
          arrayStride: attributesPerBuffer * typeInfo.size,
          stepMode: i === 0 ? 'instance' : 'vertex',
          attributes: Array(attributesPerBuffer)
            .fill(0)
            .map((_, i) => ({
              shaderLocation: currAttribute++,
              offset: i * typeInfo.size,
              format: typeInfo.format,
            })),
        });
      }
    }

    // Offset the range checks for gl_VertexIndex in the shader if we use BaseVertex
    let vertexIndexOffset = 0;
    if (p.drawCallTestParameter === 'baseVertex') {
      vertexIndexOffset += p.errorScale;
    }

    // Construct pipeline that outputs a red fragment, only if we notice any invalid values
    const vertexModule = t.makeShaderModule('vertex', {
      glsl: `
      #version 450
      ${layoutStr}

      bool valid(float f) {
        return ${validValues.map(v => `f == ${v}`).join(' || ')};
      }

      bool validationFunc(${p.type} v) {
        ${typeInfo.validationFunc}
      }

      void main() {
        bool attributesInBounds = ${attributeNames.map(a => `validationFunc(${a})`).join(' && ')};
        bool indexInBounds = gl_VertexIndex == 0 || (gl_VertexIndex >= ${vertexIndexOffset} &&
          gl_VertexIndex < ${vertexIndexOffset + numVertices});

        if (attributesInBounds && (${!p.indexed} || indexInBounds)) {
          // Success case, move the vertex out of the viewport
          gl_Position = vec4(-1.0, 0.0, 0.0, 1.0);
        } else {
          // Failure case, move the vertex inside the viewport
          gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        }
      }
    `,
    });

    const fragmentModule = t.makeShaderModule('fragment', {
      glsl: `
      #version 450
      precision mediump float;

      layout(location = 0) out vec4 fragColor;

      void main() {
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
      }
    `,
    });

    // Pipeline setup, texture setup
    const colorAttachment = t.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 1, height: 1, depth: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    const colorAttachmentView = colorAttachment.createView();

    const pipeline = t.device.createRenderPipeline({
      vertexStage: { module: vertexModule, entryPoint: 'main' },
      fragmentStage: { module: fragmentModule, entryPoint: 'main' },
      primitiveTopology: 'point-list',
      colorStates: [{ format: 'rgba8unorm', alphaBlend: {}, colorBlend: {} }],
      vertexState: {
        indexFormat: 'uint16',
        vertexBuffers,
      },
    });

    // Offset the draw call parameter we are testing by |errorScale|
    draw[p.drawCallTestParameter] += p.errorScale;

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: colorAttachmentView,
          storeOp: 'store',
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
        },
      ],
    });

    pass.setPipeline(pipeline);

    // Run the draw variant
    draw.insertInto(pass, p.indexed, p.indirect);

    pass.endPass();
    t.device.defaultQueue.submit([encoder.finish()]);

    // Validate we see green instead of red, meaning no fragment ended up on-screen
    t.expectSinglePixelIn2DTexture(
      colorAttachment,
      'rgba8unorm',
      { x: 0, y: 0 },
      { exp: new Uint8Array([0x00, 0xff, 0x00, 0xff]), layout: { mipLevel: 0 } }
    );
  });
