/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Test that variables in the shader are zero initialized`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { iterRange, unreachable } from '../../../common/util/util.js';
import { GPUTest } from '../../gpu_test.js';
import {
  kVectorContainerTypes,
  kMatrixContainerTypes,
  supportedScalarTypes,
  supportsAtomics,
} from '../types.js';

function prettyPrint(t) {
  switch (t.type) {
    case 'container':
      switch (t.containerType) {
        case 'array':
          return `array<${prettyPrint(t.elementType)}, ${t.length}>`;
        case 'struct':
          return `struct { ${t.members.map(m => prettyPrint(m)).join(', ')} }`;
        default:
          return `${t.containerType}<${prettyPrint({
            type: 'scalar',
            scalarType: t.scalarType,
            isAtomic: false,
          })}>`;
      }

      break;
    case 'scalar':
      if (t.isAtomic) {
        return `atomic<${t.scalarType}>`;
      }
      return t.scalarType;
  }
}

export const g = makeTestGroup(GPUTest);
g.test('compute,zero_init')
  .desc(
    `Test that uninitialized variables in workgroup, private, and function storage classes are initialized to zero.`
  )
  .params(u =>
    u
      // Only workgroup, function, and private variables can be declared without data bound to them.
      // The implementation's shader translator should ensure these values are initialized.
      .combine('addressSpace', ['workgroup', 'private', 'function'])
      .expand('workgroupSize', ({ addressSpace }) => {
        switch (addressSpace) {
          case 'workgroup':
            return [
              [1, 1, 1],
              [1, 32, 1],
              [64, 1, 1],
              [1, 1, 48],
              [1, 47, 1],
              [33, 1, 1],
              [1, 1, 63],
              [8, 8, 2],
              [7, 7, 3],
            ];

          case 'function':
          case 'private':
            return [[1, 1, 1]];
        }
      })
      .beginSubcases()
      // Fewer subcases: Only 0 and 2. If double-nested containers work, single-nested should too.
      .combine('_containerDepth', [0, 2])
      .expandWithParams(function* (p) {
        const kElementCounts = [
          [], // Not used. Depth 0 is always scalars.
          [1, 3, 67], // Test something above the workgroup size.
          [1, 3],
        ];

        const kMemberCounts = [1, 3];

        const memoizedTypes = [];

        function generateTypesMemo(depth) {
          if (memoizedTypes[depth] === undefined) {
            memoizedTypes[depth] = Array.from(generateTypes(depth));
          }
          return memoizedTypes[depth];
        }

        function* generateTypes(depth) {
          if (depth === 0) {
            for (const isAtomic of supportsAtomics({
              ...p,
              access: 'read_write',
              storageMode: undefined,
              containerType: 'scalar',
            })
              ? [true, false]
              : [false]) {
              for (const scalarType of supportedScalarTypes({ isAtomic, ...p })) {
                // Fewer subcases: For nested types, skip atomic u32 and non-atomic i32.
                if (p._containerDepth > 0) {
                  if (scalarType === 'u32' && isAtomic) continue;
                  if (scalarType === 'i32' && !isAtomic) continue;
                }

                yield {
                  type: 'scalar',
                  scalarType,
                  isAtomic,
                };
                if (!isAtomic) {
                  // Vector types
                  for (const vectorType of kVectorContainerTypes) {
                    // Fewer subcases: For nested types, only include
                    // vec2<u32>, vec3<i32>, and vec4<f32>
                    if (p._containerDepth > 0) {
                      if (
                        !(
                          (vectorType === 'vec2' && scalarType === 'u32') ||
                          (vectorType === 'vec3' && scalarType === 'i32') ||
                          (vectorType === 'vec4' && scalarType === 'f32')
                        )
                      ) {
                        continue;
                      }
                    }
                    yield {
                      type: 'container',
                      containerType: vectorType,
                      scalarType,
                    };
                  }
                  // Matrices can only be f32.
                  if (scalarType === 'f32') {
                    for (const matrixType of kMatrixContainerTypes) {
                      yield {
                        type: 'container',
                        containerType: matrixType,
                        scalarType,
                      };
                    }
                  }
                }
              }
            }
            return;
          }

          for (const containerType of ['array', 'struct']) {
            const innerTypes = generateTypesMemo(depth - 1);
            switch (containerType) {
              case 'array':
                for (const elementCount of kElementCounts[depth]) {
                  for (const innerType of innerTypes) {
                    yield {
                      type: 'container',
                      containerType,
                      elementType: innerType,
                      length: elementCount,
                    };
                  }
                }
                break;
              case 'struct':
                for (const memberCount of kMemberCounts) {
                  const memberIndices = new Array(memberCount);
                  for (let m = 0; m < memberCount; ++m) {
                    memberIndices[m] = m;
                  }

                  // Don't generate all possible combinations of inner struct members,
                  // because that's in the millions. Instead, just round-robin through
                  // to pick member types. Loop through the types, concatenated forward
                  // and backward, three times to produce a bounded but variable set of
                  // types.
                  const memberTypes = [...innerTypes, ...[...innerTypes].reverse()];
                  const seenTypes = new Set();
                  let typeIndex = 0;
                  while (typeIndex < memberTypes.length * 3) {
                    const prevTypeIndex = typeIndex;
                    const members = [];
                    for (const m of memberIndices) {
                      members[m] = memberTypes[typeIndex % memberTypes.length];
                      typeIndex += 1;
                    }

                    const t = {
                      type: 'container',
                      containerType,
                      members,
                    };
                    const serializedT = prettyPrint(t);
                    if (seenTypes.has(serializedT)) {
                      // We produced an identical type. shuffle the member indices,
                      // "revert" typeIndex back to where it was before this loop, and
                      // shift it by one. This helps ensure we don't loop forever, and
                      // that we produce a different type on the next iteration.
                      memberIndices.push(memberIndices.shift());
                      typeIndex = prevTypeIndex + 1;
                      continue;
                    }
                    seenTypes.add(serializedT);
                    yield t;
                  }
                }
                break;
            }
          }
        }

        for (const t of generateTypesMemo(p._containerDepth)) {
          yield {
            shaderTypeParam: prettyPrint(t),
            _type: t,
          };
        }
      })
  )
  .batch(15)
  .fn(t => {
    let moduleScope = `
      struct Output {
        failed : atomic<u32>
      }
      @group(0) @binding(0) var<storage, read_write> output : Output;

      // This uniform value that's a zero is used to prevent the shader compilers from trying to
      // unroll the massive loops generated by these tests.
      @group(0) @binding(1) var<uniform> zero : u32;
    `;
    let functionScope = '';

    const declaredStructTypes = new Map();
    const typeDecl = (function ensureType(typeName, type, depth = 0) {
      switch (type.type) {
        case 'container':
          switch (type.containerType) {
            case 'array':
              return `array<${ensureType(
                `${typeName}_ArrayElement`,
                type.elementType,
                depth + 1
              )}, ${type.length}>`;
            case 'struct': {
              if (declaredStructTypes.has(type)) {
                return declaredStructTypes.get(type);
              }

              const members = type.members
                .map((member, i) => {
                  return `\n    member${i} : ${ensureType(
                    `${typeName}_Member${i}`,
                    member,
                    depth + 1
                  )},`;
                })
                .join('');
              declaredStructTypes.set(type, typeName);
              moduleScope += `\nstruct ${typeName} {`;
              moduleScope += members;
              moduleScope += '\n};';

              return typeName;
            }
            default:
              return `${type.containerType}<${ensureType(
                typeName,
                {
                  type: 'scalar',
                  scalarType: type.scalarType,
                  isAtomic: false,
                },
                depth + 1
              )}>`;
          }

          break;
        case 'scalar':
          return type.isAtomic ? `atomic<${type.scalarType}>` : type.scalarType;
      }
    })('TestType', t.params._type);

    switch (t.params.addressSpace) {
      case 'workgroup':
      case 'private':
        moduleScope += `\nvar<${t.params.addressSpace}> testVar: ${typeDecl};`;
        break;
      case 'function':
        functionScope += `\nvar testVar: ${typeDecl};`;
        break;
    }

    const checkZeroCode = (function checkZero(value, type, depth = 0) {
      switch (type.type) {
        case 'container':
          switch (type.containerType) {
            case 'array':
              return `\nfor (var i${depth} = 0u; i${depth} < ${
                type.length
              }u + zero; i${depth} = i${depth} + 1u) {
                ${checkZero(`${value}[i${depth}]`, type.elementType, depth + 1)}
              }`;
            case 'struct':
              return type.members
                .map((member, i) => {
                  return checkZero(`${value}.member${i}`, member, depth + 1);
                })
                .join('\n');
            default:
              if (type.containerType.indexOf('vec') !== -1) {
                const length = type.containerType[3];
                return `\nfor (var i${depth} = 0u; i${depth} < ${length}u + zero; i${depth} = i${depth} + 1u) {
                  ${checkZero(
                    `${value}[i${depth}]`,
                    {
                      type: 'scalar',
                      scalarType: type.scalarType,
                      isAtomic: false,
                    },
                    depth + 1
                  )}
                }`;
              } else if (type.containerType.indexOf('mat') !== -1) {
                const cols = type.containerType[3];
                const rows = type.containerType[5];
                return `\nfor (var c${depth} = 0u; c${depth} < ${cols}u + zero; c${depth} = c${depth} + 1u) {
                  for (var r${depth} = 0u; r${depth} < ${rows}u; r${depth} = r${depth} + 1u) {
                    ${checkZero(
                      `${value}[c${depth}][r${depth}]`,
                      {
                        type: 'scalar',
                        scalarType: type.scalarType,
                        isAtomic: false,
                      },
                      depth + 1
                    )}
                  }
                }`;
              } else {
                unreachable();
              }
          }

          break;
        case 'scalar': {
          let expected;
          switch (type.scalarType) {
            case 'bool':
              expected = 'false';
              break;
            case 'f32':
              expected = '0.0';
              break;
            case 'i32':
              expected = '0';
              break;
            case 'u32':
              expected = '0u';
              break;
          }

          if (type.isAtomic) {
            value = `atomicLoad(&${value})`;
          }

          // Note: this could have an early return, but we omit it because it makes
          // the tests fail cause with DXGI_ERROR_DEVICE_HUNG on Windows.
          return `\nif (${value} != ${expected}) { atomicStore(&output.failed, 1u); }`;
        }
      }
    })('testVar', t.params._type);

    const wgsl = `
      ${moduleScope}
      @compute @workgroup_size(${t.params.workgroupSize})
      fn main() {
        ${functionScope}
        ${checkZeroCode}
        _ = zero;
      }
    `;

    if (t.params.addressSpace === 'workgroup') {
      // Populate the maximum amount of workgroup memory with known values to
      // ensure initialization overrides in another shader.
      const wg_memory_limits = t.device.limits.maxComputeWorkgroupStorageSize;
      const wg_x_dim = t.device.limits.maxComputeWorkgroupSizeX;

      const wgsl = `
      @group(0) @binding(0) var<storage, read> inputs : array<u32>;
      @group(0) @binding(1) var<storage, read_write> outputs : array<u32>;
      var<workgroup> wg_mem : array<u32, ${wg_memory_limits} / 4>;

      @compute @workgroup_size(${wg_x_dim})
      fn fill(@builtin(local_invocation_index) lid : u32) {
        const num_u32_per_invocation = ${wg_memory_limits} / (4 * ${wg_x_dim});

        for (var i = 0u; i < num_u32_per_invocation; i++) {
          let idx = num_u32_per_invocation * lid + i;
          wg_mem[idx] = inputs[idx];
        }
        workgroupBarrier();
        // Copy out to avoid wg_mem being elided.
        for (var i = 0u; i < num_u32_per_invocation; i++) {
          let idx = num_u32_per_invocation * lid + i;
          outputs[idx] = wg_mem[idx];
        }
      }
      `;

      const fillLayout = t.device.createBindGroupLayout({
        entries: [
          {
            binding: 0,
            visibility: GPUShaderStage.COMPUTE,
            buffer: { type: 'read-only-storage' },
          },
          {
            binding: 1,
            visibility: GPUShaderStage.COMPUTE,
            buffer: { type: 'storage' },
          },
        ],
      });

      const fillPipeline = t.device.createComputePipeline({
        layout: t.device.createPipelineLayout({ bindGroupLayouts: [fillLayout] }),
        label: 'Workgroup Fill Pipeline',
        compute: {
          module: t.device.createShaderModule({
            code: wgsl,
          }),
          entryPoint: 'fill',
        },
      });

      const inputBuffer = t.makeBufferWithContents(
        new Uint32Array([...iterRange(wg_memory_limits / 4, x => 0xdeadbeef)]),
        GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
      );

      t.trackForCleanup(inputBuffer);
      const outputBuffer = t.device.createBuffer({
        size: wg_memory_limits,
        usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
      });
      t.trackForCleanup(outputBuffer);

      const bg = t.device.createBindGroup({
        layout: fillPipeline.getBindGroupLayout(0),
        entries: [
          {
            binding: 0,
            resource: {
              buffer: inputBuffer,
            },
          },
          {
            binding: 1,
            resource: {
              buffer: outputBuffer,
            },
          },
        ],
      });

      const e = t.device.createCommandEncoder();
      const p = e.beginComputePass();
      p.setPipeline(fillPipeline);
      p.setBindGroup(0, bg);
      p.dispatchWorkgroups(1);
      p.end();
      t.queue.submit([e.finish()]);
    }

    const pipeline = t.device.createComputePipeline({
      layout: 'auto',
      compute: {
        module: t.device.createShaderModule({
          code: wgsl,
        }),
        entryPoint: 'main',
      },
    });

    const resultBuffer = t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
    });
    t.trackForCleanup(resultBuffer);

    const zeroBuffer = t.device.createBuffer({
      size: 4,
      usage: GPUBufferUsage.UNIFORM,
    });
    t.trackForCleanup(zeroBuffer);

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        {
          binding: 0,
          resource: {
            buffer: resultBuffer,
          },
        },
        {
          binding: 1,
          resource: {
            buffer: zeroBuffer,
          },
        },
      ],
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(1);
    pass.end();
    t.queue.submit([encoder.finish()]);
    t.expectGPUBufferValuesEqual(resultBuffer, new Uint32Array([0]));
  });
