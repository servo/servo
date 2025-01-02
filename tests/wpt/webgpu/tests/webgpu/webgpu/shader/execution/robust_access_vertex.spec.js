/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test vertex attributes behave correctly (no crash / data leak) when accessed out of bounds

Test coverage:

The following is parameterized (all combinations tested):

1) Draw call type? (drawIndexed, drawIndirect, drawIndexedIndirect)
  - Run the draw call using an index buffer and/or an indirect buffer.
  - Doesn't test direct draw, as vertex buffer OOB are CPU validated and treated as validation errors.
  - Also the instance step mode vertex buffer OOB are CPU validated for drawIndexed, so we only test
    robustness access for vertex step mode vertex buffers.

2) Draw call parameter (vertexCount, firstVertex, indexCount, firstIndex, baseVertex, instanceCount,
   vertexCountInIndexBuffer)
  - The parameter which goes out of bounds. Filtered depending on the draw call type.
  - vertexCount, firstVertex: used for drawIndirect only, test for vertex step mode buffer OOB
  - instanceCount: used for both drawIndirect and drawIndexedIndirect, test for instance step mode buffer OOB
  - baseVertex, vertexCountInIndexBuffer: used for both drawIndexed and drawIndexedIndirect, test
    for vertex step mode buffer OOB. vertexCountInIndexBuffer indicates how many vertices are used
    within the index buffer, i.e. [0, 1, ..., vertexCountInIndexBuffer-1].
  - indexCount, firstIndex: used for drawIndexedIndirect only, validate the vertex buffer access
    when the vertex itself is OOB in index buffer. This never happens in drawIndexed as we have index
    buffer OOB CPU validation for it.

3) Attribute type (float32, float32x2, float32x3, float32x4)
  - The input attribute type in the vertex shader

4) Error scale (0, 1, 4, 10^2, 10^4, 10^6)
  - Offset to add to the correct draw call parameter
  - 0 For control case

5) Additional vertex buffers (0, +4)
  - Tests that no OOB occurs if more vertex buffers are used

6) Partial last number and offset vertex buffer (false, true)
  - Tricky cases that make vertex buffer OOB.
  - With partial last number enabled, vertex buffer size will be 1 byte less than enough, making the
    last vertex OOB with 1 byte.
  - Offset vertex buffer will bind the vertex buffer to render pass with 4 bytes offset, causing OOB
  - For drawIndexed, these two flags are suppressed for instance step mode vertex buffer to make sure
    it pass the CPU validation.

The tests have one instance step mode vertex buffer bound for instanced attributes, to make sure
instanceCount / firstInstance are tested.

The tests include multiple attributes per vertex buffer.

The vertex buffers are filled by repeating a few values randomly chosen for each test until the
end of the buffer.

The tests run a render pipeline which verifies the following:
1) All vertex attribute values occur in the buffer or are 0 (for control case it can't be 0)
2) All gl_VertexIndex values are within the index buffer or 0

TODO:
Currently firstInstance is not tested, as for drawIndexed it is CPU validated, and for drawIndirect
and drawIndexedIndirect it should always be 0. Once there is an extension to allow making them non-zero,
it should be added into drawCallTestParameter list.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';
import { GPUTest, TextureTestMixin } from '../../gpu_test.js';

// Encapsulates a draw call (either indexed or non-indexed)
class DrawCall {



  // Add a float offset when binding vertex buffer


  // Keep instance step mode vertex buffer in range, in order to test vertex step
  // mode buffer OOB in drawIndexed. Setting true will suppress partialLastNumber
  // and offsetVertexBuffer for instance step mode vertex buffer.


  // Draw



  // DrawIndexed
  // For generating index buffer in drawIndexed and drawIndexedIndirect
  // For accessing index buffer in drawIndexed and drawIndexedIndirect



  // Both Draw and DrawIndexed



  constructor({
    test,
    vertexArrays,
    vertexCount,
    partialLastNumber,
    offsetVertexBuffer,
    keepInstanceStepModeBufferInRange







  }) {
    this.test = test;

    // Default arguments (valid call)
    this.vertexCount = vertexCount;
    this.firstVertex = 0;
    this.vertexCountInIndexBuffer = vertexCount;
    this.indexCount = vertexCount;
    this.firstIndex = 0;
    this.baseVertex = 0;
    this.instanceCount = vertexCount;
    this.firstInstance = 0;

    this.offsetVertexBuffer = offsetVertexBuffer;
    this.keepInstanceStepModeBufferInRange = keepInstanceStepModeBufferInRange;

    // Since vertexInIndexBuffer is mutable, generation of the index buffer should be deferred to right before calling draw

    // Generate vertex buffer
    this.vertexBuffers = vertexArrays.map((v, i) => {
      if (i === 0 && keepInstanceStepModeBufferInRange) {
        // Suppress partialLastNumber for the first vertex buffer, aka the instance step mode buffer
        return this.generateVertexBuffer(v, false);
      } else {
        return this.generateVertexBuffer(v, partialLastNumber);
      }
    });
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
    // Generate index buffer
    const indexArray = new Uint32Array(this.vertexCountInIndexBuffer).map((_, i) => i);
    const indexBuffer = this.test.makeBufferWithContents(indexArray, GPUBufferUsage.INDEX);
    this.bindVertexBuffers(pass);
    pass.setIndexBuffer(indexBuffer, 'uint32');
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
    // Generate index buffer
    const indexArray = new Uint32Array(this.vertexCountInIndexBuffer).map((_, i) => i);
    const indexBuffer = this.test.makeBufferWithContents(indexArray, GPUBufferUsage.INDEX);
    this.bindVertexBuffers(pass);
    pass.setIndexBuffer(indexBuffer, 'uint32');
    pass.drawIndexedIndirect(this.generateIndexedIndirectBuffer(), 0);
  }

  // Bind all vertex buffers generated
  bindVertexBuffers(pass) {
    let currSlot = 0;
    for (let i = 0; i < this.vertexBuffers.length; i++) {
      if (i === 0 && this.keepInstanceStepModeBufferInRange) {
        // Keep the instance step mode buffer in range
        pass.setVertexBuffer(currSlot++, this.vertexBuffers[i], 0);
      } else {
        pass.setVertexBuffer(currSlot++, this.vertexBuffers[i], this.offsetVertexBuffer ? 4 : 0);
      }
    }
  }

  // Create a vertex buffer from |vertexArray|
  // If |partialLastNumber| is true, delete one byte off the end
  generateVertexBuffer(vertexArray, partialLastNumber) {
    let size = vertexArray.byteLength;
    let length = vertexArray.length;
    if (partialLastNumber) {
      size -= 1; // Shave off one byte from the buffer size.
      length -= 1; // And one whole element from the writeBuffer.
    }
    const buffer = this.test.createBufferTracked({
      size,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST // Ensure that buffer can be used by writeBuffer
    });
    this.test.device.queue.writeBuffer(buffer, 0, vertexArray.slice(0, length));
    return buffer;
  }

  // Create an indirect buffer containing draw call values
  generateIndirectBuffer() {
    const indirectArray = new Int32Array([
    this.vertexCount,
    this.instanceCount,
    this.firstVertex,
    this.firstInstance]
    );
    return this.test.makeBufferWithContents(indirectArray, GPUBufferUsage.INDIRECT);
  }

  // Create an indirect buffer containing indexed draw call values
  generateIndexedIndirectBuffer() {
    const indirectArray = new Int32Array([
    this.indexCount,
    this.instanceCount,
    this.firstIndex,
    this.baseVertex,
    this.firstInstance]
    );
    return this.test.makeBufferWithContents(indirectArray, GPUBufferUsage.INDIRECT);
  }
}

// Parameterize different sized types






const typeInfoMap = {
  float32: {
    wgslType: 'f32',
    sizeInBytes: 4,
    validationFunc: 'return valid(v);'
  },
  float32x2: {
    wgslType: 'vec2<f32>',
    sizeInBytes: 8,
    validationFunc: 'return valid(v.x) && valid(v.y);'
  },
  float32x3: {
    wgslType: 'vec3<f32>',
    sizeInBytes: 12,
    validationFunc: 'return valid(v.x) && valid(v.y) && valid(v.z);'
  },
  float32x4: {
    wgslType: 'vec4<f32>',
    sizeInBytes: 16,
    validationFunc: `return (valid(v.x) && valid(v.y) && valid(v.z) && valid(v.w)) ||
                            (v.x == 0.0 && v.y == 0.0 && v.z == 0.0 && (v.w == 0.0 || v.w == 1.0));`
  }
};

class F extends TextureTestMixin(GPUTest) {
  generateBufferContents(
  numVertices,
  attributesPerBuffer,
  typeInfo,
  arbitraryValues,
  bufferCount)
  {
    // Make an array big enough for the vertices, attributes, and size of each element
    const vertexArray = new Float32Array(
      numVertices * attributesPerBuffer * (typeInfo.sizeInBytes / 4)
    );

    for (let i = 0; i < vertexArray.length; ++i) {
      vertexArray[i] = arbitraryValues[i % arbitraryValues.length];
    }

    // Only the first buffer is instance step mode, all others are vertex step mode buffer
    assert(bufferCount >= 2);
    const bufferContents = [];
    for (let i = 0; i < bufferCount; i++) {
      bufferContents.push(vertexArray);
    }

    return bufferContents;
  }

  generateVertexBufferDescriptors(
  bufferCount,
  attributesPerBuffer,
  format)
  {
    const typeInfo = typeInfoMap[format];
    // Vertex buffer descriptors
    const buffers = [];
    {
      let currAttribute = 0;
      for (let i = 0; i < bufferCount; i++) {
        buffers.push({
          arrayStride: attributesPerBuffer * typeInfo.sizeInBytes,
          stepMode: i === 0 ? 'instance' : 'vertex',
          attributes: Array(attributesPerBuffer).
          fill(0).
          map((_, i) => ({
            shaderLocation: currAttribute++,
            offset: i * typeInfo.sizeInBytes,
            format
          }))
        });
      }
    }
    return buffers;
  }

  generateVertexShaderCode({
    bufferCount,
    attributesPerBuffer,
    validValues,
    typeInfo,
    vertexIndexOffset,
    numVertices,
    isIndexed








  }) {
    // Create layout and attributes listing
    let layoutStr = 'struct Attributes {';
    const attributeNames = [];
    {
      let currAttribute = 0;
      for (let i = 0; i < bufferCount; i++) {
        for (let j = 0; j < attributesPerBuffer; j++) {
          layoutStr += `@location(${currAttribute}) a_${currAttribute} : ${typeInfo.wgslType},\n`;
          attributeNames.push(`a_${currAttribute}`);
          currAttribute++;
        }
      }
    }
    layoutStr += '};';

    const vertexShaderCode = `
      ${layoutStr}

      fn valid(f : f32) -> bool {
        return ${validValues.map((v) => `f == ${v}.0`).join(' || ')};
      }

      fn validationFunc(v : ${typeInfo.wgslType}) -> bool {
        ${typeInfo.validationFunc}
      }

      @vertex fn main(
        @builtin(vertex_index) VertexIndex : u32,
        attributes : Attributes
        ) -> @builtin(position) vec4<f32> {
        var attributesInBounds = ${attributeNames.
    map((a) => `validationFunc(attributes.${a})`).
    join(' && ')};

        var indexInBoundsCountFromBaseVertex =
            (VertexIndex >= ${vertexIndexOffset}u &&
            VertexIndex < ${vertexIndexOffset + numVertices}u);
        var indexInBounds = VertexIndex == 0u || indexInBoundsCountFromBaseVertex;

        var Position : vec4<f32>;
        if (attributesInBounds && (${!isIndexed} || indexInBounds)) {
          // Success case, move the vertex to the right of the viewport to show that at least one case succeed
          Position = vec4<f32>(0.5, 0.0, 0.0, 1.0);
        } else {
          // Failure case, move the vertex to the left of the viewport
          Position = vec4<f32>(-0.5, 0.0, 0.0, 1.0);
        }
        return Position;
      }`;
    return vertexShaderCode;
  }

  createRenderPipeline({
    bufferCount,
    attributesPerBuffer,
    validValues,
    typeInfo,
    vertexIndexOffset,
    numVertices,
    isIndexed,
    buffers









  }) {
    const pipeline = this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: this.generateVertexShaderCode({
            bufferCount,
            attributesPerBuffer,
            validValues,
            typeInfo,
            vertexIndexOffset,
            numVertices,
            isIndexed
          })
        }),
        entryPoint: 'main',
        buffers
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }`
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'point-list' }
    });
    return pipeline;
  }

  doTest({
    bufferCount,
    attributesPerBuffer,
    dataType,
    validValues,
    vertexIndexOffset,
    numVertices,
    isIndexed,
    isIndirect,
    drawCall










  }) {
    // Vertex buffer descriptors
    const buffers = this.generateVertexBufferDescriptors(
      bufferCount,
      attributesPerBuffer,
      dataType
    );

    // Pipeline setup, texture setup
    const pipeline = this.createRenderPipeline({
      bufferCount,
      attributesPerBuffer,
      validValues,
      typeInfo: typeInfoMap[dataType],
      vertexIndexOffset,
      numVertices,
      isIndexed,
      buffers
    });

    const colorAttachment = this.createTextureTracked({
      format: 'rgba8unorm',
      size: { width: 2, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });
    const colorAttachmentView = colorAttachment.createView();

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachmentView,
        clearValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);

    // Run the draw variant
    drawCall.insertInto(pass, isIndexed, isIndirect);

    pass.end();
    this.device.queue.submit([encoder.finish()]);

    // Validate we see green on the left pixel, showing that no failure case is detected
    this.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
    { coord: { x: 0, y: 0 }, exp: new Uint8Array([0x00, 0xff, 0x00, 0xff]) }]
    );
  }
}

export const g = makeTestGroup(F);

g.test('vertex_buffer_access').
params(
  (u) =>
  u.
  combineWithParams([
  { indexed: false, indirect: true },
  { indexed: true, indirect: false },
  { indexed: true, indirect: true }]
  ).
  expand('drawCallTestParameter', function* (p) {
    if (p.indexed) {
      yield* ['baseVertex', 'vertexCountInIndexBuffer'];
      if (p.indirect) {
        yield* ['indexCount', 'instanceCount', 'firstIndex'];
      }
    } else if (p.indirect) {
      yield* ['vertexCount', 'instanceCount', 'firstVertex'];
    }
  }).
  combine('type', Object.keys(typeInfoMap)).
  combine('additionalBuffers', [0, 4]).
  combine('partialLastNumber', [false, true]).
  combine('offsetVertexBuffer', [false, true]).
  beginSubcases().
  combine('errorScale', [0, 1, 4, 10 ** 2, 10 ** 4, 10 ** 6]).
  unless((p) => p.drawCallTestParameter === 'instanceCount' && p.errorScale > 10 ** 4) // To avoid timeout
).
fn((t) => {
  const p = t.params;
  const typeInfo = typeInfoMap[p.type];

  // Number of vertices to draw
  const numVertices = 4;
  // Each buffer is bound to this many attributes (2 would mean 2 attributes per buffer)
  const attributesPerBuffer = 2;
  // Some arbitrary values to fill our buffer with to avoid collisions with other tests
  const arbitraryValues = [990, 685, 446, 175];

  // A valid value is 0 or one in the buffer
  const validValues =
  p.errorScale === 0 && !p.offsetVertexBuffer && !p.partialLastNumber ?
  arbitraryValues // Control case with no OOB access, must read back valid values in buffer
  : [0, ...arbitraryValues]; // Testing case with OOB access, can be 0 for OOB data

  // Generate vertex buffer contents. Only the first buffer is instance step mode, all others are vertex step mode
  const bufferCount = p.additionalBuffers + 2; // At least one instance step mode and one vertex step mode buffer
  const bufferContents = t.generateBufferContents(
    numVertices,
    attributesPerBuffer,
    typeInfo,
    arbitraryValues,
    bufferCount
  );

  // Mutable draw call
  const draw = new DrawCall({
    test: t,
    vertexArrays: bufferContents,
    vertexCount: numVertices,
    partialLastNumber: p.partialLastNumber,
    offsetVertexBuffer: p.offsetVertexBuffer,
    keepInstanceStepModeBufferInRange: p.indexed && !p.indirect // keep instance step mode buffer in range for drawIndexed
  });

  // Offset the draw call parameter we are testing by |errorScale|
  draw[p.drawCallTestParameter] += p.errorScale;
  // Offset the range checks for gl_VertexIndex in the shader if we use BaseVertex
  let vertexIndexOffset = 0;
  if (p.drawCallTestParameter === 'baseVertex') {
    vertexIndexOffset += p.errorScale;
  }

  t.doTest({
    bufferCount,
    attributesPerBuffer,
    dataType: p.type,
    validValues,
    vertexIndexOffset,
    numVertices,
    isIndexed: p.indexed,
    isIndirect: p.indirect,
    drawCall: draw
  });
});