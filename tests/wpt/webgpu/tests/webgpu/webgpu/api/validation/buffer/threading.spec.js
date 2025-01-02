/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
TODO:
- Try to map on one thread while {pending, mapped, mappedAtCreation, mappedAtCreation+unmap+mapped}
  on another thread.
- Invalid to postMessage a mapped range's ArrayBuffer or ArrayBufferView
  {with, without} it being in the transfer array.
- Copy GPUBuffer to another thread while {pending, mapped mappedAtCreation} on {same,diff} thread
  (valid), then try to map on that thread (invalid)
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);