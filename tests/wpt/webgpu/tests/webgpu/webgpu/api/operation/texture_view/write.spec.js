/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test the result of writing textures through texture views with various options.

All x= every possible view write method: {
  - storage write {fragment, compute}
  - render pass store
  - render pass resolve
}

Format reinterpretation is not tested here. It is in format_reinterpretation.spec.ts.

TODO: Write helper for this if not already available (see resource_init, buffer_sync_test for related code).
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('format').
desc(
  `Views of every allowed format.

- x= every texture format
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
`
).
unimplemented();

g.test('dimension').
desc(
  `Views of every allowed dimension.

- x= a representative subset of formats
- x= {every texture dimension} x {every valid view dimension}
  (per gpuweb#79 no dimension-count reinterpretations, like 2d-array <-> 3d, are possible)
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
`
).
unimplemented();

g.test('aspect').
desc(
  `Views of every allowed aspect of depth/stencil textures.

- x= every depth/stencil format
- x= {"all", "stencil-only", "depth-only"} where valid for the format
- x= sampleCount {1, 4} if valid
- x= every possible view write method (see above)
`
).
unimplemented();