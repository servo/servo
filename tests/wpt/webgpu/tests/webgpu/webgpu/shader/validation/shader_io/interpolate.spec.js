/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for the interpolate attribute`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { generateShader } from './util.js';

export const g = makeTestGroup(ShaderValidationTest);

// List of valid interpolation attributes.
const kValidInterpolationAttributes = new Set([
  '',
  '@interpolate(flat)',
  '@interpolate(perspective)',
  '@interpolate(perspective, center)',
  '@interpolate(perspective, centroid)',
  '@interpolate(perspective, sample)',
  '@interpolate(linear)',
  '@interpolate(linear, center)',
  '@interpolate(linear, centroid)',
  '@interpolate(linear, sample)',
]);

g.test('type_and_sampling')
  .desc(`Test that all combinations of interpolation type and sampling are validated correctly.`)
  .params(u =>
    u
      .combine('stage', ['vertex', 'fragment'])
      .combine('io', ['in', 'out'])
      .combine('use_struct', [true, false])
      .combine('type', [
        '',
        'flat',
        'perspective',
        'linear',
        'center', // Invalid as first param
        'centroid', // Invalid as first param
        'sample', // Invalid as first param
      ])
      .combine('sampling', [
        '',
        'center',
        'centroid',
        'sample',
        'flat', // Invalid as second param
        'perspective', // Invalid as second param
        'linear', // Invalid as second param
      ])
      .beginSubcases()
  )
  .fn(t => {
    if (t.params.stage === 'vertex' && t.params.use_struct === false) {
      t.skip('vertex output must include a position builtin, so must use a struct');
    }

    let interpolate = '';
    if (t.params.type !== '' || t.params.sampling !== '') {
      interpolate = '@interpolate(';
      if (t.params.type !== '') {
        interpolate += `${t.params.type}`;
      }
      if (t.params.sampling !== '') {
        interpolate += `, ${t.params.sampling}`;
      }
      interpolate += `)`;
    }
    const code = generateShader({
      attribute: '@location(0)' + interpolate,
      type: 'f32',
      stage: t.params.stage,
      io: t.params.io,
      use_struct: t.params.use_struct,
    });

    t.expectCompileResult(kValidInterpolationAttributes.has(interpolate), code);
  });

g.test('require_location')
  .desc(`Test that the interpolate attribute is only accepted with user-defined IO.`)
  .params(u =>
    u
      .combine('stage', ['vertex', 'fragment'])
      .combine('attribute', ['@location(0)', '@builtin(position)'])
      .combine('use_struct', [true, false])
      .beginSubcases()
  )
  .fn(t => {
    if (
      t.params.stage === 'vertex' &&
      t.params.use_struct === false &&
      !t.params.attribute.includes('position')
    ) {
      t.skip('vertex output must include a position builtin, so must use a struct');
    }

    const code = generateShader({
      attribute: t.params.attribute + `@interpolate(flat)`,
      type: 'vec4<f32>',
      stage: t.params.stage,
      io: t.params.stage === 'fragment' ? 'in' : 'out',
      use_struct: t.params.use_struct,
    });
    t.expectCompileResult(t.params.attribute === '@location(0)', code);
  });

g.test('integral_types')
  .desc(`Test that the implementation requires @interpolate(flat) for integral user-defined IO.`)
  .params(u =>
    u
      .combine('stage', ['vertex', 'fragment'])
      .combine('type', ['i32', 'u32', 'vec2<i32>', 'vec4<u32>'])
      .combine('use_struct', [true, false])
      .combine('attribute', kValidInterpolationAttributes)
      .beginSubcases()
  )
  .fn(t => {
    if (t.params.stage === 'vertex' && t.params.use_struct === false) {
      t.skip('vertex output must include a position builtin, so must use a struct');
    }

    const code = generateShader({
      attribute: '@location(0)' + t.params.attribute,
      type: t.params.type,
      stage: t.params.stage,
      io: t.params.stage === 'vertex' ? 'out' : 'in',
      use_struct: t.params.use_struct,
    });

    t.expectCompileResult(t.params.attribute === '@interpolate(flat)', code);
  });

g.test('duplicate')
  .desc(`Test that the interpolate attribute can only be applied once.`)
  .params(u => u.combine('attr', ['', '@interpolate(flat)']))
  .fn(t => {
    const code = generateShader({
      attribute: `@location(0) @interpolate(flat) ${t.params.attr}`,
      type: 'vec4<f32>',
      stage: 'fragment',
      io: 'in',
      use_struct: false,
    });
    t.expectCompileResult(t.params.attr === '', code);
  });

const kValidationTests = {
  valid: {
    src: `@interpolate(flat)`,
    pass: true,
  },
  no_space: {
    src: `@interpolate(perspective,center)`,
    pass: true,
  },
  trailing_comma_one_arg: {
    src: `@interpolate(flat,)`,
    pass: true,
  },
  trailing_comma_two_arg: {
    src: `@interpolate(perspective, center,)`,
    pass: true,
  },
  newline: {
    src: '@\ninterpolate(flat)',
    pass: true,
  },
  comment: {
    src: `@/* comment */interpolate(flat)`,
    pass: true,
  },

  no_params: {
    src: `@interpolate()`,
    pass: false,
  },
  missing_left_paren: {
    src: `@interpolate flat)`,
    pass: false,
  },
  missing_value_and_left_paren: {
    src: `@interpolate)`,
    pass: false,
  },
  missing_right_paren: {
    src: `@interpolate(flat`,
    pass: false,
  },
  missing_parens: {
    src: `@interpolate`,
    pass: false,
  },
  missing_comma: {
    src: `@interpolate(perspective center)`,
    pass: false,
  },
  numeric: {
    src: `@interpolate(1)`,
    pass: false,
  },
  numeric_second_param: {
    src: `@interpolate(perspective, 1)`,
    pass: false,
  },
};

g.test('interpolation_validation')
  .desc(`Test validation of interpolation`)
  .params(u => u.combine('attr', keysOf(kValidationTests)))
  .fn(t => {
    const code = `
@vertex fn main(${kValidationTests[t.params.attr].src} @location(0) b: f32) ->
    @builtin(position) vec4<f32> {
  return vec4f(0);
}`;
    t.expectCompileResult(kValidationTests[t.params.attr].pass, code);
  });
