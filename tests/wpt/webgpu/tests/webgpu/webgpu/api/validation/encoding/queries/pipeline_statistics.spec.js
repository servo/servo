/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation for encoding pipeline statistics queries.
Excludes query begin/end balance and nesting (begin_end.spec.ts)
and querySet/queryIndex (general.spec.ts).

TODO: pipeline statistics queries are removed from core; consider moving tests to another suite.
TODO:
- Test pipelineStatistics with {undefined, empty, duplicated, full (control case)} values
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);
