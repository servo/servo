/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Test primitive topology rendering.

Draw a primitive using 6 vertices with each topology and check if the pixel is covered.

Vertex sequence and coordinates are the same for each topology:
  - Vertex buffer = [v1, v2, v3, v4, v5, v6]
  - Topology = [point-list, line-list, line-strip, triangle-list, triangle-strip]

Test locations are framebuffer coordinates:
  - Pixel value { valid: green, invalid: black, format: 'rgba8unorm'}
  - Test point is valid if the pixel value equals the covered pixel value at the test location.
  - Primitive restart occurs for strips (line-strip and triangle-strip) between [v3, v4].

  Topology: point-list         Valid test location(s)           Invalid test location(s)

       v2    v4     v6         Every vertex.                    Line-strip locations.
                                                                Triangle-list locations.
                                                                Triangle-strip locations.

   v1     v3     v5

  Topology: line-list (3 lines)

       v2    v4     v6         Center of three line segments:   Line-strip locations.
      *      *      *          {v1,V2}, {v3,v4}, and {v4,v5}.   Triangle-list locations.
     *      *      *                                            Triangle-strip locations.
    *      *      *
   v1     v3     v5

  Topology: line-strip (5 lines)

       v2    v4     v6
       **    **     *
      *  *  *  *   *           Line-list locations              Triangle-list locations.
     *    **     **          + Center of two line segments:     Triangle-strip locations.
    v1    v3     v5            {v2,v3} and {v4,v5}.
                                                                With primitive restart:
                                                                Line segment {v3, v4}.

  Topology: triangle-list (2 triangles)

      v2       v4    v6
      **        ******         Center of two triangle(s):       Triangle-strip locations.
     ****        ****          {v1,v2,v3} and {v4,v5,v6}.
    ******        **
   v1     v3      v5

  Topology: triangle-strip (4 triangles)

      v2        v4      v6
      ** ****** ** ******      Triangle-list locations          None.
     **** **** **** ****     + Center of two triangle(s):
    ****** ** ****** **        {v2,v3,v4} and {v3,v4,v5}.       With primitive restart:
   v1       v3        v5                                        Triangle {v2, v3, v4}
                                                                and {v3, v4, v5}.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';

const kRTSize = 56;
const kColorFormat = 'rgba8unorm';
const kValidPixelColor = new Uint8Array([0x00, 0xff, 0x00, 0xff]); // green
const kInvalidPixelColor = new Uint8Array([0x00, 0x00, 0x00, 0x00]); // black

class Point2D {
  constructor(x, y) {
    this.x = x;
    this.y = y;
    this.z = 0;
    this.w = 1;
  }

  toNDC() {
    // NDC coordinate space is y-up, so we negate the y mapping.
    // To ensure the resulting vertex in NDC will be placed at the center of the pixel, we
    // must offset by the pixel coordinates or 0.5.
    return new Point2D((2 * (this.x + 0.5)) / kRTSize - 1, (-2 * (this.y + 0.5)) / kRTSize + 1);
  }

  static getMidpoint(a, b) {
    return new Point2D((a.x + b.x) / 2, (a.y + b.y) / 2);
  }

  static getCentroid(a, b, c) {
    return new Point2D((a.x + b.x + c.x) / 3, (a.y + b.y + c.y) / 3);
  }
}

const VertexLocations = [
  new Point2D(8, 24), // v1
  new Point2D(16, 8), // v2
  new Point2D(24, 24), // v3
  new Point2D(32, 8), // v4
  new Point2D(40, 24), // v5
  new Point2D(48, 8), // v6
];

function getPointTestLocations(expectedColor) {
  // Test points are always equal to vertex locations.
  const testLocations = [];
  for (const location of VertexLocations) {
    testLocations.push({ coord: location, exp: expectedColor });
  }
  return testLocations;
}

function getLineTestLocations(expectedColor) {
  // Midpoints of 3 line segments
  return [
    {
      // Line {v1, v2}
      coord: Point2D.getMidpoint(VertexLocations[0], VertexLocations[1]),
      exp: expectedColor,
    },
    {
      // Line {v3, v4}
      coord: Point2D.getMidpoint(VertexLocations[2], VertexLocations[3]),
      exp: expectedColor,
    },
    {
      // Line {v5, v6}
      coord: Point2D.getMidpoint(VertexLocations[4], VertexLocations[5]),
      exp: expectedColor,
    },
  ];
}

function getPrimitiveRestartLineTestLocations(expectedColor) {
  // Midpoints of 2 line segments
  return [
    {
      // Line {v1, v2}
      coord: Point2D.getMidpoint(VertexLocations[0], VertexLocations[1]),
      exp: expectedColor,
    },
    {
      // Line {v5, v6}
      coord: Point2D.getMidpoint(VertexLocations[4], VertexLocations[5]),
      exp: expectedColor,
    },
  ];
}

function getLineStripTestLocations(expectedColor) {
  // Midpoints of 2 line segments
  return [
    {
      // Line {v2, v3}
      coord: Point2D.getMidpoint(VertexLocations[1], VertexLocations[2]),
      exp: expectedColor,
    },
    {
      // Line {v4, v5}
      coord: Point2D.getMidpoint(VertexLocations[3], VertexLocations[4]),
      exp: expectedColor,
    },
  ];
}

function getTriangleListTestLocations(expectedColor) {
  // Center of two triangles
  return [
    {
      // Triangle {v1, v2, v3}
      coord: Point2D.getCentroid(VertexLocations[0], VertexLocations[1], VertexLocations[2]),
      exp: expectedColor,
    },
    {
      // Triangle {v4, v5, v6}
      coord: Point2D.getCentroid(VertexLocations[3], VertexLocations[4], VertexLocations[5]),
      exp: expectedColor,
    },
  ];
}

function getTriangleStripTestLocations(expectedColor) {
  // Center of two triangles
  return [
    {
      // Triangle {v2, v3, v4}
      coord: Point2D.getCentroid(VertexLocations[1], VertexLocations[2], VertexLocations[3]),
      exp: expectedColor,
    },
    {
      // Triangle {v3, v4, v5}
      coord: Point2D.getCentroid(VertexLocations[2], VertexLocations[3], VertexLocations[4]),
      exp: expectedColor,
    },
  ];
}

function getDefaultTestLocations({
  topology,
  primitiveRestart = false,
  invalidateLastInList = false,
}) {
  function maybeInvalidateLast(locations) {
    if (!invalidateLastInList) return locations;

    return locations.map((tl, i) => {
      if (i === locations.length - 1) {
        return {
          coord: tl.coord,
          exp: kInvalidPixelColor,
        };
      } else {
        return tl;
      }
    });
  }

  let testLocations;
  switch (topology) {
    case 'point-list':
      testLocations = [
        ...getPointTestLocations(kValidPixelColor),
        ...getLineStripTestLocations(kInvalidPixelColor),
        ...getTriangleListTestLocations(kInvalidPixelColor),
        ...getTriangleStripTestLocations(kInvalidPixelColor),
      ];

      break;
    case 'line-list':
      testLocations = [
        ...maybeInvalidateLast(getLineTestLocations(kValidPixelColor)),
        ...getLineStripTestLocations(kInvalidPixelColor),
        ...getTriangleListTestLocations(kInvalidPixelColor),
        ...getTriangleStripTestLocations(kInvalidPixelColor),
      ];

      break;
    case 'line-strip':
      testLocations = [
        ...(primitiveRestart
          ? getPrimitiveRestartLineTestLocations(kValidPixelColor)
          : getLineTestLocations(kValidPixelColor)),
        ...getLineStripTestLocations(kValidPixelColor),
        ...getTriangleListTestLocations(kInvalidPixelColor),
        ...getTriangleStripTestLocations(kInvalidPixelColor),
      ];

      break;
    case 'triangle-list':
      testLocations = [
        ...maybeInvalidateLast(getTriangleListTestLocations(kValidPixelColor)),
        ...getTriangleStripTestLocations(kInvalidPixelColor),
      ];

      break;
    case 'triangle-strip':
      testLocations = [
        ...getTriangleListTestLocations(kValidPixelColor),
        ...getTriangleStripTestLocations(primitiveRestart ? kInvalidPixelColor : kValidPixelColor),
      ];

      break;
  }

  return testLocations;
}

function generateVertexBuffer(vertexLocations) {
  const vertexCoords = new Float32Array(vertexLocations.length * 4);
  for (let i = 0; i < vertexLocations.length; i++) {
    const point = vertexLocations[i].toNDC();
    vertexCoords[i * 4 + 0] = point.x;
    vertexCoords[i * 4 + 1] = point.y;
    vertexCoords[i * 4 + 2] = point.z;
    vertexCoords[i * 4 + 3] = point.w;
  }
  return vertexCoords;
}

const kDefaultDrawCount = 6;
class PrimitiveTopologyTest extends TextureTestMixin(GPUTest) {
  makeAttachmentTexture() {
    return this.device.createTexture({
      format: kColorFormat,
      size: { width: kRTSize, height: kRTSize, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    });
  }

  run({
    topology,
    indirect,
    testLocations,
    primitiveRestart = false,
    drawCount = kDefaultDrawCount,
  }) {
    const colorAttachment = this.makeAttachmentTexture();

    // Color load operator will clear color attachment to zero.
    const encoder = this.device.createCommandEncoder();
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
        {
          view: colorAttachment.createView(),
          clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });

    let stripIndexFormat = undefined;
    if (topology === 'triangle-strip' || topology === 'line-strip') {
      stripIndexFormat = 'uint32';
    }

    // Draw a primitive using 6 vertices based on the type.
    // Pixels are generated based on vertex position.
    // If point, 1 pixel is generated at each vertex location.
    // Otherwise, >1 pixels could be generated.
    // Output color is solid green.
    renderPass.setPipeline(
      this.device.createRenderPipeline({
        layout: 'auto',
        vertex: {
          module: this.device.createShaderModule({
            code: `
              @vertex fn main(
                @location(0) pos : vec4<f32>
                ) -> @builtin(position) vec4<f32> {
                return pos;
              }`,
          }),
          entryPoint: 'main',
          buffers: [
            {
              arrayStride: 4 * Float32Array.BYTES_PER_ELEMENT,
              attributes: [
                {
                  format: 'float32x4',
                  offset: 0,
                  shaderLocation: 0,
                },
              ],
            },
          ],
        },
        fragment: {
          module: this.device.createShaderModule({
            code: `
              @fragment fn main() -> @location(0) vec4<f32> {
                return vec4<f32>(0.0, 1.0, 0.0, 1.0);
              }`,
          }),
          entryPoint: 'main',
          targets: [{ format: kColorFormat }],
        },
        primitive: {
          topology,
          stripIndexFormat,
        },
      })
    );

    // Create vertices for the primitive in a vertex buffer and bind it.
    const vertexCoords = generateVertexBuffer(VertexLocations);
    const vertexBuffer = this.makeBufferWithContents(vertexCoords, GPUBufferUsage.VERTEX);
    renderPass.setVertexBuffer(0, vertexBuffer);

    // Restart the strip between [v3, <restart>, v4].
    if (primitiveRestart) {
      const indexBuffer = this.makeBufferWithContents(
        new Uint32Array([0, 1, 2, -1, 3, 4, 5]),
        GPUBufferUsage.INDEX
      );

      renderPass.setIndexBuffer(indexBuffer, 'uint32');

      if (indirect) {
        renderPass.drawIndexedIndirect(
          this.makeBufferWithContents(
            new Uint32Array([drawCount + 1, 1, 0, 0, 0]),
            GPUBufferUsage.INDIRECT
          ),

          0
        );
      } else {
        renderPass.drawIndexed(drawCount + 1); // extra index for restart
      }
    } else {
      if (indirect) {
        renderPass.drawIndirect(
          this.makeBufferWithContents(
            new Uint32Array([drawCount, 1, 0, 0]),
            GPUBufferUsage.INDIRECT
          ),

          0
        );
      } else {
        renderPass.draw(drawCount);
      }
    }

    renderPass.end();

    this.device.queue.submit([encoder.finish()]);
    this.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, testLocations);
  }
}

export const g = makeTestGroup(PrimitiveTopologyTest);

const topologies = ['point-list', 'line-list', 'line-strip', 'triangle-list', 'triangle-strip'];

g.test('basic')
  .desc(
    `Compute test locations for valid and invalid pixels for each topology.
  If the primitive covers the pixel, the color value will be |kValidPixelColor|.
  Otherwise, a non-covered pixel will be |kInvalidPixelColor|.

  Params:
    - topology= {...all topologies}
    - indirect= {true, false}
    - primitiveRestart= { true, false } - always false for non-strip topologies
  `
  )
  .params(u =>
    u //
      .combine('topology', topologies)
      .combine('indirect', [false, true])
      .combine('primitiveRestart', [false, true])
      .unless(
        p => p.primitiveRestart && p.topology !== 'line-strip' && p.topology !== 'triangle-strip'
      )
  )
  .fn(t => {
    t.run({
      ...t.params,
      testLocations: getDefaultTestLocations(t.params),
    });
  });

g.test('unaligned_vertex_count')
  .desc(
    `Test that drawing with a number of vertices that's not a multiple of the vertices a given primitive list topology is not an error. The last primitive is not drawn.

    Params:
    - topology= {line-list, triangle-list}
    - indirect= {true, false}
    - drawCount - number of vertices to draw. A value smaller than the test's default of ${kDefaultDrawCount}.
                   One smaller for line-list. One or two smaller for triangle-list.
    `
  )
  .params(u =>
    u //
      .combine('topology', ['line-list', 'triangle-list'])
      .combine('indirect', [false, true])
      .expand('drawCount', function* (p) {
        switch (p.topology) {
          case 'line-list':
            yield kDefaultDrawCount - 1;
            break;
          case 'triangle-list':
            yield kDefaultDrawCount - 1;
            yield kDefaultDrawCount - 2;
            break;
        }
      })
  )
  .fn(t => {
    const testLocations = getDefaultTestLocations({ ...t.params, invalidateLastInList: true });
    t.run({
      ...t.params,
      testLocations,
    });
  });
