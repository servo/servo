/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
'storageBarrier' affects memory and atomic operations in the storage address space.

All synchronization functions execute a control barrier with Acquire/Release memory ordering.
That is, all synchronization functions, and affected memory and atomic operations are ordered
in program order relative to the synchronization function. Additionally, the affected memory
and atomic operations program-ordered before the synchronization function must be visible to
all other threads in the workgroup before any affected memory or atomic operation program-ordered
after the synchronization function is executed by a member of the workgroup. All synchronization
functions use the Workgroup memory scope. All synchronization functions have a Workgroup
execution scope.

All synchronization functions must only be used in the compute shader stage.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('stage')
  .specURL('https://www.w3.org/TR/WGSL/#sync-builtin-functions')
  .desc(
    `
All synchronization functions must only be used in the compute shader stage.
`
  )
  .params(u => u.combine('stage', ['vertex', 'fragment', 'compute']))
  .unimplemented();

g.test('barrier')
  .specURL('https://www.w3.org/TR/WGSL/#sync-builtin-functions')
  .desc(
    `
fn storageBarrier()
`
  )
  .unimplemented();
