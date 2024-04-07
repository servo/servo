"""Generates Canvas tests from YAML file definitions."""
# Current code status:
#
# This was originally written by Philip Taylor for use at
# http://philip.html5.org/tests/canvas/suite/tests/
#
# It has been adapted for use with the Web Platform Test Suite suite at
# https://github.com/web-platform-tests/wpt/
#
# The original version had a number of now-removed features (multiple versions
# of each test case of varying verbosity, Mozilla mochitests, semi-automated
# test harness). It also had a different directory structure.

# To update or add test cases:
#
# * Modify the tests*.yaml files.
#  - 'name' is an arbitrary hierarchical name to help categorise tests.
#  - 'desc' is a rough description of what behaviour the test aims to test.
#  - 'code' is JavaScript code to execute, with some special commands starting
#    with '@'.
#  - 'expected' is what the final canvas output should be: a string 'green' or
#    'clear' (100x50 images in both cases), or a string 'size 100 50' (or any
#    other size) followed by Python code using Pycairo to generate the image.
#
# * Run "./build.sh".
# This requires a few Python modules which might not be ubiquitous.
# It will usually emit some warnings, which ideally should be fixed but can
# generally be safely ignored.
#
# * Test the tests, add new ones to Git, remove deleted ones from Git, etc.

from typing import Any, DefaultDict, FrozenSet, List, Mapping, MutableMapping
from typing import Optional, Set, Tuple

import re
import collections
import dataclasses
import enum
import importlib
import itertools
import os
import pathlib
import sys
import textwrap

import jinja2

try:
    import cairocffi as cairo  # type: ignore
except ImportError:
    import cairo

try:
    # Compatible and lots faster.
    import syck as yaml  # type: ignore
except ImportError:
    import yaml


class Error(Exception):
    """Base class for all exceptions raised by this module"""


class InvalidTestDefinitionError(Error):
    """Raised on invalid test definition."""


def _double_quote_escape(string: str) -> str:
    return string.replace('\\', '\\\\').replace('"', '\\"')


def _escape_js(string: str) -> str:
    string = _double_quote_escape(string)
    # Kind of an ugly hack, for nicer failure-message output.
    string = re.sub(r'\[(\w+)\]', r'[\\""+(\1)+"\\"]', string)
    return string


def _unroll(text: str) -> str:
    """Unrolls text with all possible permutations of the parameter lists.

    Example:
    >>> print _unroll('f = {<a | b>: <1 | 2 | 3>};')
    // a
    f = {a: 1};
    f = {a: 2};
    f = {a: 3};
    // b
    f = {b: 1};
    f = {b: 2};
    f = {b: 3};
    """
    patterns = []  # type: List[Tuple[str, List[str]]]
    while True:
        match = re.search(r'<([^>]+)>', text)
        if not match:
            break
        key = f'@unroll_pattern_{len(patterns)}'
        values = text[match.start(1):match.end(1)]
        text = text[:match.start(0)] + key + text[match.end(0):]
        patterns.append((key, [value.strip() for value in values.split('|')]))

    def unroll_patterns(text: str,
                        patterns: List[Tuple[str, List[str]]],
                        label: Optional[str] = None) -> List[str]:
        if not patterns:
            return [text]
        patterns = patterns.copy()
        key, values = patterns.pop(0)
        return (['// ' + label] if label else []) + list(
            itertools.chain.from_iterable(
                unroll_patterns(text.replace(key, value), patterns, value)
                for value in values))

    result = '\n'.join(unroll_patterns(text, patterns))
    return result


def _expand_nonfinite(method: str, argstr: str, tail: str) -> str:
    """
    >>> print _expand_nonfinite('f', '<0 a>, <0 b>', ';')
    f(a, 0);
    f(0, b);
    f(a, b);
    >>> print _expand_nonfinite('f', '<0 a>, <0 b c>, <0 d>', ';')
    f(a, 0, 0);
    f(0, b, 0);
    f(0, c, 0);
    f(0, 0, d);
    f(a, b, 0);
    f(a, b, d);
    f(a, 0, d);
    f(0, b, d);
    """
    # argstr is "<valid-1 invalid1-1 invalid2-1 ...>, ..." (where usually
    # 'invalid' is Infinity/-Infinity/NaN).
    args = []
    for arg in argstr.split(', '):
        match = re.match('<(.*)>', arg)
        if match is None:
            raise InvalidTestDefinitionError(
                f'Expected arg to match format "<(.*)>", but was: {arg}')
        a = match.group(1)
        args.append(a.split(' '))
    calls = []
    # Start with the valid argument list.
    call = [args[j][0] for j in range(len(args))]
    # For each argument alone, try setting it to all its invalid values:
    for i, arg in enumerate(args):
        for a in arg[1:]:
            c2 = call[:]
            c2[i] = a
            calls.append(c2)
    # For all combinations of >= 2 arguments, try setting them to their
    # first invalid values. (Don't do all invalid values, because the
    # number of combinations explodes.)
    def f(c: List[str], start: int, depth: int) -> None:
        for i in range(start, len(args)):
            if len(args[i]) > 1:
                a = args[i][1]
                c2 = c[:]
                c2[i] = a
                if depth > 0:
                    calls.append(c2)
                f(c2, i + 1, depth + 1)

    f(call, 0, 0)

    str_calls = (', '.join(c) for c in calls)
    return '\n'.join(f'{method}({params}){tail}' for params in str_calls)


def _get_test_sub_dir(name: str, name_to_sub_dir: Mapping[str, str]) -> str:
    for prefix in sorted(name_to_sub_dir.keys(), key=len, reverse=True):
        if name.startswith(prefix):
            return name_to_sub_dir[prefix]
    raise InvalidTestDefinitionError(
        f'Test "{name}" has no defined target directory mapping')


def _remove_extra_newlines(text: str) -> str:
    """Remove newlines if a backslash is found at end of line."""
    # Lines ending with '\' gets their newline character removed.
    text = re.sub(r'\\\n', '', text, flags=re.MULTILINE | re.DOTALL)

    # Lines ending with '\-' gets their newline and any leading white spaces on
    # the following line removed.
    text = re.sub(r'\\-\n\s*', '', text, flags=re.MULTILINE | re.DOTALL)
    return text


def _expand_test_code(code: str) -> str:
    code = re.sub(r' @moz-todo', '', code)

    code = re.sub(r'@moz-UniversalBrowserRead;', '', code)

    code = _remove_extra_newlines(code)

    # Unroll expressions with a cross-product-style parameter expansion.
    code = re.sub(r'@unroll ([^;]*;)', lambda m: _unroll(m.group(1)), code)

    code = re.sub(r'@nonfinite ([^(]+)\(([^)]+)\)(.*)', lambda m:
                  _expand_nonfinite(m.group(1), m.group(2), m.group(3)),
                  code)  # Must come before '@assert throws'.

    code = re.sub(r'@assert pixel (\d+,\d+) == (\d+,\d+,\d+,\d+);',
                  r'_assertPixel(canvas, \1, \2);', code)

    code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+);',
                  r'_assertPixelApprox(canvas, \1, \2, 2);', code)

    code = re.sub(r'@assert pixel (\d+,\d+) ==~ (\d+,\d+,\d+,\d+) \+/- (\d+);',
                  r'_assertPixelApprox(canvas, \1, \2, \3);', code)

    code = re.sub(r'@assert throws (\S+_ERR) (.*?);$',
                  r'assert_throws_dom("\1", function() { \2; });', code,
                  flags=re.MULTILINE | re.DOTALL)

    code = re.sub(r'@assert throws (\S+Error) (.*?);$',
                  r'assert_throws_js(\1, function() { \2; });', code,
                  flags=re.MULTILINE | re.DOTALL)

    code = re.sub(
        r'@assert (.*) === (.*);', lambda m:
        (f'_assertSame({m.group(1)}, {m.group(2)}, '
         f'"{_escape_js(m.group(1))}", "{_escape_js(m.group(2))}");'), code)

    code = re.sub(
        r'@assert (.*) !== (.*);', lambda m:
        (f'_assertDifferent({m.group(1)}, {m.group(2)}, '
         f'"{_escape_js(m.group(1))}", "{_escape_js(m.group(2))}");'), code)

    code = re.sub(
        r'@assert (.*) =~ (.*);',
        lambda m: f'assert_regexp_match({m.group(1)}, {m.group(2)});', code)

    code = re.sub(
        r'@assert (.*);',
        lambda m: f'_assert({m.group(1)}, "{_escape_js(m.group(1))}");', code)

    assert '@' not in code

    return code


_TestParams = Mapping[str, Any]
_MutableTestParams = MutableMapping[str, Any]


class _CanvasType(str, enum.Enum):
    HTML_CANVAS = 'HtmlCanvas'
    OFFSCREEN_CANVAS = 'OffscreenCanvas'
    WORKER = 'Worker'


class _TemplateType(str, enum.Enum):
    REFERENCE = 'reference'
    HTML_REFERENCE = 'html_reference'
    TESTHARNESS = 'testharness'


@dataclasses.dataclass
class _OutputPaths:
    element: str
    offscreen: str

    def sub_path(self, sub_dir: str):
        """Create a new _OutputPaths that is a subpath of this _OutputPath."""
        return _OutputPaths(element=os.path.join(self.element, sub_dir),
                            offscreen=os.path.join(self.offscreen, sub_dir))


def _validate_test(test: _TestParams):
    if test.get('expected', '') == 'green' and re.search(
            r'@assert pixel .* 0,0,0,0;', test['code']):
        print(f'Probable incorrect pixel test in {test["name"]}')

    if 'size' in test and (not isinstance(test['size'], list)
                           or len(test['size']) != 2):
        raise InvalidTestDefinitionError(
            f'Invalid canvas size "{test["size"]}" in test {test["name"]}. '
            'Expected an array with two numbers.')

    if test['template_type'] == _TemplateType.TESTHARNESS:
        valid_test_types = {'sync', 'async', 'promise'}
    else:
        valid_test_types = {'promise'}

    test_type = test.get('test_type')
    if test_type is not None and test_type not in valid_test_types:
        raise InvalidTestDefinitionError(
            f'Invalid test_type: {test_type}. '
            f'Valid values are: {valid_test_types}.')


def _render_template(jinja_env: jinja2.Environment, template: jinja2.Template,
                     params: _TestParams) -> str:
    """Renders the specified jinja template.

    The template is repetitively rendered until no more changes are observed.
    This allows for template parameters to refer to other template parameters.
    """
    rendered = template.render(params)
    previous = ''
    while rendered != previous:
        previous = rendered
        template = jinja_env.from_string(rendered)
        rendered = template.render(params)
    return rendered


def _render(jinja_env: jinja2.Environment, template_name: str,
            params: _TestParams, output_file_name: str):
    template = jinja_env.get_template(template_name)
    file_content = _render_template(jinja_env, template, params)
    pathlib.Path(output_file_name).write_text(file_content, 'utf-8')


def _preprocess_code(jinja_env: jinja2.Environment, code: str,
                     params: _TestParams) -> str:
    code = _expand_test_code(code)
    # Render the code on its own, as it could contain templates expanding
    # to multiple lines. This is needed to get proper indentation of the
    # code in the main template.
    code = _render_template(jinja_env, jinja_env.from_string(code), params)
    return code


class _Variant():

    def __init__(self, params: _MutableTestParams) -> None:
        self._params = params

    @property
    def params(self) -> _TestParams:
        """Read-only getter for this variant's param dict."""
        return self._params

    @staticmethod
    def create_with_defaults(test: _TestParams) -> '_Variant':
        """Create a _Variant from the specified params.

        Default values are added for certain parameters, if missing."""
        params = {
            'desc': '',
            'size': [100, 50],
            'variant_names': [],
        }
        params.update(test)
        return _Variant(params)

    def _get_variant_name(self, jinja_env: jinja2.Environment) -> str:
        name = self.params['name']
        if self.params.get('append_variants_to_name', True):
            name = '.'.join([name] + self.params['variant_names'])

        name = jinja_env.from_string(name).render(self.params)
        return name

    def _get_file_name(self) -> str:
        file_name = self.params['name']

        if 'manual' in self.params:
            file_name += '-manual'

        return file_name

    def _get_canvas_types(self) -> FrozenSet[_CanvasType]:
        canvas_types = self.params.get('canvas_types', _CanvasType)
        invalid_types = {
            type
            for type in canvas_types if type not in list(_CanvasType)
        }
        if invalid_types:
            raise InvalidTestDefinitionError(
                f'Invalid canvas_types: {list(invalid_types)}. '
                f'Accepted values are: {[t.value for t in _CanvasType]}')
        return frozenset(_CanvasType(t) for t in canvas_types)

    def _get_template_type(self) -> _TemplateType:
        if 'reference' in self.params and 'html_reference' in self.params:
            raise InvalidTestDefinitionError(
                f'Test {self.params["name"]} is invalid, "reference" and '
                '"html_reference" can\'t both be specified at the same time.')

        if 'reference' in self.params:
            return _TemplateType.REFERENCE
        if 'html_reference' in self.params:
            return _TemplateType.HTML_REFERENCE
        return _TemplateType.TESTHARNESS

    def finalize_params(self, jinja_env: jinja2.Environment) -> None:
        """Finalize this variant by adding computed param fields."""
        self._params['name'] = self._get_variant_name(jinja_env)
        self._params['file_name'] = self._get_file_name()
        self._params['canvas_types'] = self._get_canvas_types()
        self._params['template_type'] = self._get_template_type()

        if 'reference' in self._params:
            self._params['reference'] = _preprocess_code(
                jinja_env, self._params['reference'], self._params)

        if 'html_reference' in self._params:
            self._params['html_reference'] = _preprocess_code(
                jinja_env, self._params['html_reference'], self._params)

        code_params = dict(self.params)
        if _CanvasType.HTML_CANVAS in self.params['canvas_types']:
            code_params['canvas_type'] = _CanvasType.HTML_CANVAS.value
            self._params['code_element'] = _preprocess_code(
                jinja_env, self._params['code'], code_params)

        if _CanvasType.OFFSCREEN_CANVAS in self.params['canvas_types']:
            code_params['canvas_type'] = _CanvasType.OFFSCREEN_CANVAS.value
            self._params['code_offscreen'] = _preprocess_code(
                jinja_env, self._params['code'], code_params)

        if _CanvasType.WORKER in self.params['canvas_types']:
            code_params['canvas_type'] = _CanvasType.WORKER.value
            self._params['code_worker'] = _preprocess_code(
                jinja_env, self._params['code'], code_params)

        _validate_test(self._params)

    def generate_expected_image(self, output_dirs: _OutputPaths) -> None:
        """Creates a reference image using Cairo and save filename in params."""
        if 'expected' not in self.params:
            return

        expected = self.params['expected']
        name = self.params['name']

        if expected == 'green':
            self._params['expected_img'] = '/images/green-100x50.png'
            return
        if expected == 'clear':
            self._params['expected_img'] = '/images/clear-100x50.png'
            return
        if ';' in expected:
            print(f'Found semicolon in {name}')
        expected = re.sub(
            r'^size (\d+) (\d+)',
            r'surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, \1, \2)'
            r'\ncr = cairo.Context(surface)', expected)

        output_paths = output_dirs.sub_path(name)
        if _CanvasType.HTML_CANVAS in self.params['canvas_types']:
            expected_canvas = (
                f'{expected}\n'
                f'surface.write_to_png("{output_paths.element}.png")\n')
            eval(compile(expected_canvas, f'<test {name}>', 'exec'), {},
                 {'cairo': cairo})

        if {_CanvasType.OFFSCREEN_CANVAS, _CanvasType.WORKER
            } & self.params['canvas_types']:
            expected_offscreen = (
                f'{expected}\n'
                f'surface.write_to_png("{output_paths.offscreen}.png")\n')
            eval(compile(expected_offscreen, f'<test {name}>', 'exec'), {},
                 {'cairo': cairo})

        self._params['expected_img'] = f'{name}.png'

    def _write_reference_test(self, jinja_env: jinja2.Environment,
                              output_files: _OutputPaths):
        params = dict(self.params)
        if _CanvasType.HTML_CANVAS in params['canvas_types']:
            _render(jinja_env, 'reftest_element.html', params,
                    f'{output_files.element}.html')
        if _CanvasType.OFFSCREEN_CANVAS in params['canvas_types']:
            _render(jinja_env, 'reftest_offscreen.html', params,
                    f'{output_files.offscreen}.html')
        if _CanvasType.WORKER in params['canvas_types']:
            _render(jinja_env, 'reftest_worker.html', params,
                    f'{output_files.offscreen}.w.html')

        params['is_test_reference'] = True
        is_html_ref = params['template_type'] == _TemplateType.HTML_REFERENCE
        ref_template = 'reftest.html' if is_html_ref else 'reftest_element.html'
        if _CanvasType.HTML_CANVAS in params['canvas_types']:
            _render(jinja_env, ref_template, params,
                    f'{output_files.element}-expected.html')
        if {_CanvasType.OFFSCREEN_CANVAS, _CanvasType.WORKER
            } & params['canvas_types']:
            _render(jinja_env, ref_template, params,
                    f'{output_files.offscreen}-expected.html')

    def _write_testharness_test(self, jinja_env: jinja2.Environment,
                                output_files: _OutputPaths):
        # Create test cases for canvas and offscreencanvas.
        if _CanvasType.HTML_CANVAS in self.params['canvas_types']:
            _render(jinja_env, 'testharness_element.html', self.params,
                    f'{output_files.element}.html')

        if _CanvasType.OFFSCREEN_CANVAS in self.params['canvas_types']:
            _render(jinja_env, 'testharness_offscreen.html', self.params,
                    f'{output_files.offscreen}.html')

        if _CanvasType.WORKER in self.params['canvas_types']:
            _render(jinja_env, 'testharness_worker.js', self.params,
                    f'{output_files.offscreen}.worker.js')

    def generate_test(self, jinja_env: jinja2.Environment,
                      output_dirs: _OutputPaths) -> None:
        """Generate the test files to the specified output dirs."""
        output_files = output_dirs.sub_path(self.params['file_name'])

        if self.params['template_type'] in (_TemplateType.REFERENCE,
                                            _TemplateType.HTML_REFERENCE):
            self._write_reference_test(jinja_env, output_files)
        else:
            self._write_testharness_test(jinja_env, output_files)


def _recursive_expand_variant_matrix(original_test: _TestParams,
                                     variant_matrix: List[_TestParams],
                                     current_selection: List[Tuple[str, Any]],
                                     test_variants: List[_Variant]):
    if len(current_selection) == len(variant_matrix):
        # Selection for each variant is done, so add a new test to test_list.
        test = dict(original_test)
        variant_name_list = []
        for variant_name, variant_params in current_selection:
            test.update(variant_params)
            variant_name_list.append(variant_name)
        # Expose variant names as a list so they can be used from the yaml
        # files, which helps with better naming of tests.
        test.update({'variant_names': variant_name_list})
        test_variants.append(_Variant.create_with_defaults(test))
    else:
        # Continue the recursion with each possible selection for the current
        # variant.
        variant = variant_matrix[len(current_selection)]
        for variant_options in variant.items():
            current_selection.append(variant_options)
            _recursive_expand_variant_matrix(original_test, variant_matrix,
                                             current_selection, test_variants)
            current_selection.pop()


def _get_variants(test: _TestParams) -> List[_Variant]:
    current_selection = []
    test_variants = []
    variants = test.get('variants', [])
    if not isinstance(variants, list):
        raise InvalidTestDefinitionError(
            textwrap.dedent("""
            Variants must be specified as a list of variant dimensions, e.g.:
              variants:
              - dimension1-variant1:
                  param: ...
                dimension1-variant2:
                  param: ...
              - dimension2-variant1:
                  param: ...
                dimension2-variant2:
                  param: ..."""))
    _recursive_expand_variant_matrix(test, variants, current_selection,
                                     test_variants)
    return test_variants


def _check_uniqueness(tested: DefaultDict[str, Set[_CanvasType]], name: str,
                      canvas_types: FrozenSet[_CanvasType]) -> None:
    already_tested = tested[name].intersection(canvas_types)
    if already_tested:
        raise InvalidTestDefinitionError(
            f'Test {name} is defined twice for types {already_tested}')
    tested[name].update(canvas_types)


def generate_test_files(name_to_dir_file: str) -> None:
    """Generate Canvas tests from YAML file definition."""
    output_dirs = _OutputPaths(element='../element', offscreen='../offscreen')

    jinja_env = jinja2.Environment(
        loader=jinja2.PackageLoader('gentestutilsunion'),
        keep_trailing_newline=True,
        trim_blocks=True,
        lstrip_blocks=True)

    jinja_env.filters['double_quote_escape'] = _double_quote_escape

    # Run with --test argument to run unit tests.
    if len(sys.argv) > 1 and sys.argv[1] == '--test':
        doctest = importlib.import_module('doctest')
        doctest.testmod()
        sys.exit()

    name_to_sub_dir = (yaml.safe_load(
        pathlib.Path(name_to_dir_file).read_text(encoding='utf-8')))

    tests = []
    test_yaml_directory = 'yaml-new'
    yaml_files = [
        os.path.join(test_yaml_directory, f)
        for f in os.listdir(test_yaml_directory) if f.endswith('.yaml')
    ]
    for t in sum([
            yaml.safe_load(pathlib.Path(f).read_text(encoding='utf-8'))
            for f in yaml_files
    ], []):
        if 'DISABLED' in t:
            continue
        if 'meta' in t:
            eval(compile(t['meta'], '<meta test>', 'exec'), {},
                 {'tests': tests})
        else:
            tests.append(t)

    # Ensure the test output directories exist.
    test_dirs = [output_dirs.element, output_dirs.offscreen]
    for sub_dir in set(name_to_sub_dir.values()):
        test_dirs.append(f'{output_dirs.element}/{sub_dir}')
        test_dirs.append(f'{output_dirs.offscreen}/{sub_dir}')
    for d in test_dirs:
        try:
            os.mkdir(d)
        except FileExistsError:
            pass  # Ignore if it already exists,

    used_tests = collections.defaultdict(set)
    for test in tests:
        print(test['name'])
        for variant in _get_variants(test):
            variant.finalize_params(jinja_env)
            if test['name'] != variant.params['name']:
                print(f'  {variant.params["name"]}')

            sub_dir = _get_test_sub_dir(variant.params['file_name'],
                                        name_to_sub_dir)
            output_sub_dirs = output_dirs.sub_path(sub_dir)

            _check_uniqueness(used_tests, variant.params['name'],
                              variant.params['canvas_types'])
            variant.generate_expected_image(output_sub_dirs)
            variant.generate_test(jinja_env, output_sub_dirs)

    print()
