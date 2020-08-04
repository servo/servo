/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `API Validation Tests for RenderPass StoreOp.

  Test Coverage Needed:

  - Test that when depthReadOnly is true, depthStoreOp must be 'store'

  - Test that when stencilReadOnly is true, stencilStoreOp must be 'store'`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';

import { ValidationTest } from './../validation_test.js';

export const g = makeTestGroup(ValidationTest);
