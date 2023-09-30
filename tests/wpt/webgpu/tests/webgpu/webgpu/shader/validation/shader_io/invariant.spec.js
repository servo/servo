/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for the invariant attribute`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { kBuiltins } from './builtins.spec.js';
import { generateShader } from './util.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  invariant: {
    src: `@invariant`,
    pass: true,
  },
  comment: {
    src: `@/* comment */invariant`,
    pass: true,
  },
  split_line: {
    src: '@\ninvariant',
    pass: true,
  },
  empty_parens: {
    src: `@invariant()`,
    pass: false,
  },
  value: {
    src: `@invariant(0)`,
    pass: false,
  },
  missing_right_paren: {
    src: `@invariant(`,
    pass: false,
  },
  missing_left_paren: {
    src: `@invariant)`,
    pass: false,
  },
  duplicate: {
    src: `@invariant @invariant`,
    pass: false,
  },
};

g.test('parsing')
  .desc(`Test parsing of the invariant attribute`)
  .params(u => u.combine('attr', keysOf(kTests)))
  .fn(t => {
    const code = `
    struct VertexOut {
      @builtin(position) ${kTests[t.params.attr].src} position : vec4<f32>
    };
    @vertex
    fn main() -> VertexOut {
      return VertexOut();
    }
    `;
    t.expectCompileResult(kTests[t.params.attr].pass, code);
  });

g.test('valid_only_with_vertex_position_builtin')
  .desc(`Test that the invariant attribute is only accepted with the vertex position builtin`)
  .params(u => u.combineWithParams(kBuiltins).combine('use_struct', [true, false]).beginSubcases())
  .fn(t => {
    const code = generateShader({
      attribute: `@builtin(${t.params.name}) @invariant`,
      type: t.params.type,
      stage: t.params.stage,
      io: t.params.io,
      use_struct: t.params.use_struct,
    });

    t.expectCompileResult(t.params.name === 'position', code);
  });

g.test('not_valid_on_user_defined_io')
  .desc(`Test that the invariant attribute is not accepted on user-defined IO attributes.`)
  .params(u => u.combine('use_invariant', [true, false]).beginSubcases())
  .fn(t => {
    const invariant = t.params.use_invariant ? '@invariant' : '';
    const code = `
    struct VertexOut {
      @location(0) ${invariant} loc0 : vec4<f32>,
      @builtin(position) position : vec4<f32>,
    };
    @vertex
    fn main() -> VertexOut {
      return VertexOut();
    }
    `;
    t.expectCompileResult(!t.params.use_invariant, code);
  });
