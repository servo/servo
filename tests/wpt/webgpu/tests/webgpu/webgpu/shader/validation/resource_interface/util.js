/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/

/** Helper function for emitting a resource declaration's group and binding attributes */
function groupAndBinding(group, binding) {
  return (
    `${group !== undefined ? `@group(${group})` : '/* no group */'} ` +
    `${binding !== undefined ? `@binding(${binding})` : '/* no binding */'}`
  );
}

/** Helper function for emitting a resource declaration for the given type */
function basicEmitter(type) {
  return (name, group, binding) => `${groupAndBinding(group, binding)} var ${name} : ${type};\n`;
}

/** Map of resource declaration name, to an emitter. */
export const kResourceEmitters = new Map([
  ['texture_1d', basicEmitter('texture_1d<i32>')],
  ['texture_2d', basicEmitter('texture_2d<i32>')],
  ['texture_2d_array', basicEmitter('texture_2d_array<f32>')],
  ['texture_3d', basicEmitter('texture_3d<i32>')],
  ['texture_cube', basicEmitter('texture_cube<u32>')],
  ['texture_cube_array', basicEmitter('texture_cube_array<u32>')],
  ['texture_multisampled_2d', basicEmitter('texture_multisampled_2d<i32>')],
  ['texture_external', basicEmitter('texture_external')],
  ['texture_storage_1d', basicEmitter('texture_storage_1d<rgba8unorm, write>')],
  ['texture_storage_2d', basicEmitter('texture_storage_2d<rgba8sint, write>')],
  ['texture_storage_2d_array', basicEmitter('texture_storage_2d_array<r32uint, write>')],
  ['texture_storage_3d', basicEmitter('texture_storage_3d<rg32uint, write>')],
  ['texture_depth_2d', basicEmitter('texture_depth_2d')],
  ['texture_depth_2d_array', basicEmitter('texture_depth_2d_array')],
  ['texture_depth_cube', basicEmitter('texture_depth_cube')],
  ['texture_depth_cube_array', basicEmitter('texture_depth_cube_array')],
  ['texture_depth_multisampled_2d', basicEmitter('texture_depth_multisampled_2d')],
  ['sampler', basicEmitter('sampler')],
  ['sampler_comparison', basicEmitter('sampler_comparison')],
  [
    'uniform',
    (name, group, binding) =>
      `${groupAndBinding(group, binding)} var<uniform> ${name} : array<vec4<f32>, 16>;\n`,
  ],

  [
    'storage',
    (name, group, binding) =>
      `${groupAndBinding(group, binding)} var<storage> ${name} : array<vec4<f32>, 16>;\n`,
  ],
]);

/** A small selection of resource declaration names, which can be used in test permutations */
export const kResourceKindsA = ['storage', 'texture_2d', 'texture_external', 'uniform'];

/** A small selection of resource declaration names, which can be used in test permutations */
export const kResourceKindsB = ['texture_3d', 'texture_storage_1d', 'uniform'];

/** An enumerator of shader stages */

/**
 * declareEntrypoint emits the WGSL to declare an entry point with the given name, stage and body.
 * The generated function will have an appropriate return type and return statement, so that @p body
 * does not have to change between stage.
 * @param name the entry point function name
 * @param stage the entry point stage
 * @param body the body of the function (excluding any automatically suffixed return statements)
 * @returns the WGSL string for the entry point
 */
export function declareEntrypoint(name, stage, body) {
  switch (stage) {
    case 'vertex':
      return `@vertex
fn ${name}() -> @builtin(position) vec4f {
  ${body}
  return vec4f();
}`;
    case 'fragment':
      return `@fragment
fn ${name}() {
  ${body}
}`;
    case 'compute':
      return `@compute @workgroup_size(1)
fn ${name}() {
  ${body}
}`;
  }
}
