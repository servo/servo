/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for valid operations with various client-side thread-shared state of GPUBuffers.

States to test:
- mapping pending
- mapped
- mapped at creation
- mapped at creation, then unmapped
- mapped at creation, then unmapped, then re-mapped
- destroyed

TODO: Look for more things to test.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('serialize')
  .desc(
    `Copy a GPUBuffer to another thread while it is in various states on
{the sending thread, yet another thread}.`
  )
  .unimplemented();

g.test('destroyed')
  .desc(`Destroy on one thread while in various states in another thread.`)
  .unimplemented();
