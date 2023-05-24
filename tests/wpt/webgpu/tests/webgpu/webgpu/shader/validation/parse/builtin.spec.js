/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for @builtin`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kTests = {
  pos: {
    src: `@builtin(position)`,
    pass: true,
  },
  trailing_comma: {
    src: `@builtin(position,)`,
    pass: true,
  },
  newline_in_attr: {
    src: `@ \n builtin(position)`,
    pass: true,
  },
  whitespace_in_attr: {
    src: `@/* comment */builtin/* comment */\n\n(\t/*comment*/position/*comment*/)`,
    pass: true,
  },
  invalid_name: {
    src: `@abuiltin(position)`,
    pass: false,
  },
  no_params: {
    src: `@builtin`,
    pass: false,
  },
  missing_param: {
    src: `@builtin()`,
    pass: false,
  },
  missing_parens: {
    src: `@builtin position`,
    pass: false,
  },
  missing_lparen: {
    src: `@builtin position)`,
    pass: false,
  },
  missing_rparen: {
    src: `@builtin(position`,
    pass: false,
  },
  multiple_params: {
    src: `@builtin(position, frag_depth)`,
    pass: false,
  },
  ident_param: {
    src: `@builtin(identifier)`,
    pass: false,
  },
  number_param: {
    src: `@builtin(2)`,
    pass: false,
  },
};

g.test('parse')
  .desc(`Test that @builtin is parsed correctly.`)
  .params(u => u.combine('builtin', keysOf(kTests)))
  .fn(t => {
    const src = kTests[t.params.builtin].src;
    const code = `
@vertex
fn main() -> ${src} vec4<f32> {
  return vec4<f32>(.4, .2, .3, .1);
}`;
    t.expectCompileResult(kTests[t.params.builtin].pass, code);
  });

g.test('placement')
  .desc('Tests the locations @builtin is allowed to appear')
  .params(u =>
    u
      .combine('scope', [
        // The fn-param and fn-ret are part of the shader_io/builtins tests
        'private-var',
        'storage-var',
        'struct-member',
        'non-ep-param',
        'non-ep-ret',
        'fn-decl',
        'fn-var',
        'while-stmt',
        undefined,
      ])
      .combine('attribute', [
        {
          'private-var': false,
          'storage-var': false,
          'struct-member': true,
          'non-ep-param': false,
          'non-ep-ret': false,
          'fn-decl': false,
          'fn-var': false,
          'fn-return': false,
          'while-stmt': false,
        },
      ])
      .beginSubcases()
  )
  .fn(t => {
    const scope = t.params.scope;

    const attr = '@builtin(vertex_index)';
    const code = `
      ${scope === 'private-var' ? attr : ''}
      var<private> priv_var : u32;

      ${scope === 'storage-var' ? attr : ''}
      @group(0) @binding(0)
      var<storage> stor_var : u32;

      struct A {
        ${scope === 'struct-member' ? attr : ''}
        a : u32,
      }

      fn v(${scope === 'non-ep-param' ? attr : ''} i : u32) ->
            ${scope === 'non-ep-ret' ? attr : ''} u32 { return 1; }

      @vertex
      ${scope === 'fn-decl' ? attr : ''}
      fn f(
        @location(0) b : u32,
      ) -> @builtin(position) vec4f {
        ${scope === 'fn-var' ? attr : ''}
        var<function> func_v : u32;

        ${scope === 'while-stmt' ? attr : ''}
        while false {}

        return vec4(1, 1, 1, 1);
      }
    `;

    t.expectCompileResult(scope === undefined || t.params.attribute[scope], code);
  });
