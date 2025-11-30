Canvas test generator (gentest.sh)
==================================

The script gentest.sh is used to generate canvas WPT tests, found under
wpt/html/canvas.

# Purpose for generating canvas tests
Generating tests for the canvas API has multiple advantages. It allows
generating lots of tests with minimal boilerplate and configuration. In
particular:

 - Canvas tests all have common boilerplate, like defining a whole HTML page,
   creating a canvas and reading back pixels. The code we care about is usually
   only a few lines of JavaScript. By using a test generator, we can write tests
   focussing on these few relevant lines, abstracting away all of the
   boilerplate needed to run these lines.

 - Canvas exists in multiple flavors (HTMLCanvasElement, OffscreenCanvas) and
   can run in different environments (main thread, worker). Using a code
   generator allows tests to be implemented only once and run then in all the
   flavors or environment we need test coverage for.

 - Canvas rendering can be affected by a large number of states. Implementations
   can have different code paths for different permutations of these states. For
   instance, simply testing that a rectangle is correctly drawn requires
   validating different permutations of whether the canvas has an alpha channel,
   whether layers are used, whether the context uses a globalAlpha, which
   globalCompositeOperation is used, which filters are used, whether shadows are
   enabled, and so on. Bugs occurring only for some specific combinations of
   these states have been found. A test generator allows for easily creating
   a large number of tests, or tests validating a large number of variant
   permutations, all with minimal boilerplate.

# Running gentest.sh

You can generate canvas tests by running `wpt update-built --include canvas`, or
by running `gentest.sh` directly:
 - Make a python virtual environment somewhere (it doesn't matter where):

    `python3 -m venv venv`

 - Enter the virtual environment:

    `source venv/bin/activate`

 - This script depends on the `cairocffi`, `jinja2` and `pyyaml` Python
   packages. You can install them using [`requirements_build.txt`](
   https://github.com/web-platform-tests/wpt/blob/master/tools/ci/requirements_build.txt):

    `python3 -m pip install -r tools/ci/requirements_build.txt`

 - Change to the directory with this script and run it:

    `python3 gentest.py`

See [WPT documentation](
https://web-platform-tests.org/running-tests/from-local-system.html#system-setup
) for the current minimal Python version required. If you modify `gentest.py`,
it's recommended to use that exact Python version to avoid accidentally using
new Python features that aren't be supported by that minimal version.
[pyenv](https://github.com/pyenv/pyenv) can be used instead of the `venv`
approach described above, to pin the `html/canvas/tools` folder to that exact
Python version, without impacting the rest of the system. For instance:
```shell
pyenv install 3.8
cd html/canvas/tools
pyenv local 3.8
python3 -m pip install -r $WPT_CHECKOUT/tools/ci/requirements_build.txt
python3 gentest.py
```

# Canvas test definition

The tests are defined in YAML files, found in `wpt/html/canvas/tools/yaml`. The
YAML definition files consists of a sequence of dictionaries, each with at a
minimum the keys `name:` and `code:`. For instance:

```yaml
- name: 2d.sample.draws-red
  code: |
    ctx.fillStyle = 'red';
    ctx.fillRect(0, 0, 10, 10);
    @assert pixel 5,5 == 255,0,0,255;

- name: 2d.sample.draws-green
  code: |
    ctx.fillStyle = 'green';
    ctx.fillRect(0, 0, 10, 10);
    @assert pixel 5,5 == 0,255,0,255;
```

From this configuration, the test generator would produce multiple test files
and fill-in the boilerplate needed to run these JavaScript lines.

See the constants `_TEST_DEFINITION_PARAMS` and `_GENERATED_PARAMS` in the
`gentest.sh` for a full list and description of the available parameters.

## Jinja templating
The test generator uses Jinja templates to generate the different test files it
produces. The templates can be found under `wpt/html/canvas/tools/templates`.
When rendering templates, Jinja uses a dictionary of parameters to lookup
variables referred to by the template. In the test generator, this dictionary is
actually the YAML dictionary defining the test itself.

Take for instance the test:
```yaml
- name: 2d.sample.draws-red
  code: |
    ctx.fillStyle = 'red';
    ctx.fillRect(0, 0, 10, 10);
    @assert pixel 5,5 == 255,0,0,255;
```
In the template `.../templates/testharness_element.html`, the title of the
generated HTML is defined as:
```html
<title>Canvas test: {{ name }}</title>
```

When rendering this template, Jinja looks-up the `name:` key from the YAML test
definition, which in the example above would be `2d.sample.draws-red`, producing
this HTML result:
```html
<title>Canvas test: 2d.sample.draws-red</title>
```

Now, more interestingly, all the parameter values in the test definition are
also Jinja templates. They get rendered on demand, before being used by Jinja
into another template. Since all of these use the test's YAML definition as
param dictionary, test parameters can refer to each others:

```yaml
- name: 2d.sample.draws-red
  expected_color: 255,0,0,{{ alpha }}
  alpha: 255
  code: |
    ctx.fillStyle = 'red';
    ctx.fillRect(0, 0, 10, 10);
    @assert pixel 5,5 == {{ expected_color }};
```

All the test parameters are also registered as templates loadable from other
Jinja templates, with `{% import ... %}` statements for instance. This can be
useful to organize the test definition and allow reuse of Jinja statements. For
instance:
```yaml
- name: 2d.sample.macros
  macros: |
    {% macro rgba_format(color) %}
      {% set r, g, b, a = color -%}
      rgba{{ (r, g, b, a) -}}
    {% endmacro %}

    {% macro assert_format(color) %}
      {% set r, g, b, a = color -%}
      {{- '%d,%d,%d,%d' | format(r, g, b, a * 255) -}}
    {% endmacro %}
  code: |
    {% import 'macros' as m %}
    ctx.fillStyle = '{{ m.rgba_format(color) }}';
    ctx.fillRect(0, 0, 10, 10);
    @assert pixel 5,5 == {{ m.assert_format(color) }};
  color: [64, 128, 192, 1.0]
```

These types of parameterization might seem strange and overkill in toy examples
like these, but it's in fact really useful when using the `variants:` feature
([more on this below](#test-variants)).

## Canvas Types

By default, the generator produces three flavors of each tests, one for each of
three different canvas types:
 - An `HTMLCanvasElement`.
 - An `OffscreenCanvas`, used in a main thread script.
 - An `OffscreenCanvas`, used in a worker.

`HTMLCanvasElement` tests get generated into the folder `.../canvas/element`,
while the main thread and worker `OffscreenCanvas` tests are generated in the
folder `.../canvas/offscreen`.

Some tests are specific to certain canvas types. The canvas types to be
generated can be specified by setting the `canvas_types:` config to a list with
one or many of the following strings:
 - `'HtmlCanvas'`
 - `'OffscreenCanvas'`
 - `'Worker'`

For instance:
```yaml
- name: 2d.sample.offscreen-specific
  canvas_types: ['OffscreenCanvas', 'Worker']
  code: |
    assert_not_equals(canvas.convertToBlob(), null);
```

## JavaScript tests (testharness.js)

The test generator can generate both JavaScript tests (`testharness.js`), or
Reftests. By default, the generator produces [JavaScript tests](
https://web-platform-tests.org/writing-tests/testharness.html). These are
implemented with the [testharness.js library](
https://web-platform-tests.org/writing-tests/testharness-api.html). Assertions
must be used to determine whether they succeed. Standard assertions provided by
`testharness.js` can be used, like `assert_true`, `assert_equals`, etc.

### Canvas specific helpers
Canvas tests also have access to additional assertion types and other helpers
defined in `wpt/html/canvas/resources/canvas-tests.js`. Most of these however
are private and meant to be used via macros provided by this test generator
(denoted by the character "@"). Note that these "@" macros are implemented as
regexp-replace, so their syntax is very strict (e.g. they don't tolerate extra
whitespaces and some reserve `;` as terminating character).

 - `@assert pixel x,y == r,g,b,a;`

   Asserts that the color at the pixel position `[x, y]` exactly equals the RGBA
   values `[r, g, b, a]`.

 - `@assert pixel x,y ==~ r,g,b,a;`

   Asserts that the color at the pixel position `[x, y]` approximately equals
   the RGBA values `[r, g, b, a]`, within +/- 2.

 - `@assert pixel x,y ==~ r,g,b,a +/- t;`

   Asserts that the color at the pixel position `[x, y]` approximately equals
   the RGBA values `[r, g, b, a]`, within +/- `t` for each individual channel.

 - `@assert throws *_ERR code;`

   Shorthand for `assert_throws_dom`, running `code` and verifying that it
   throws a DOM exception `*_ERR` (e.g. `INDEX_SIZE_ERR`).

 - `@assert throws *Error code;`

   Shorthand for `assert_throws_js`, running `code` and verifying that it throws
   a JavaScript exception `*Error` (e.g. `TypeError`).

 - `@assert actual === expected;`

   Shorthand for `assert_equals`, asserting that `actual` is the same as
   `expected`.

 - `@assert actual !== expected;`

   Shorthand for `assert_not_equals`, asserting that `actual` is different than
   `expected`.

 - `@assert actual =~ expected;`

   Shorthand for `assert_regexp_match`, asserting that `actual` matches the
   regular expression `expected`.

 - `@assert cond;`

   Shorthand for `assert_true`, but evaluating `cond` as a boolean by prefixing
   it with `!!`.

### JavaScript test types
`testharness.js` allows the creation of synchronous, asynchronous or promise
tests (see [here](
https://web-platform-tests.org/writing-tests/testharness-api.html#defining-tests
) for details).

To choose what test types to generate, set the `test_type` parameter to one of:
 - `sync`
 - `async`
 - `promise`

For instance, a synchronous test would use `test_type: sync`:

```yaml
- name: 2d.sample.sync-test
  desc: Example synchronous test
  canvas_types: ['HtmlCanvas']
  test_type: sync
  code: |
    assert_regexp_match(canvas.toDataURL(), /^data:/);
```

Given this config, the code generator would generate an `HTMLCanvasElement` test
with the following `<script>` tag (not showing the rest of the HTML file).
``` JavaScript
test(t => {
  var canvas = document.getElementById('c');
  var ctx = canvas.getContext('2d');

  assert_regexp_match(canvas.toDataURL(), /^data:/);

}, "Example synchronous test");
```

To test asynchronous code, `test_type: async` can be use as in:
```yaml
- name: 2d.sample.async-test
  desc: Example asynchronous test
  canvas_types: ['HtmlCanvas']
  test_type: async
  code: |
    canvas.toBlob(t.step_func_done(blob => {
      assert_greater_than(blob.size, 0);
    }));
```
``` JavaScript
async_test(t => {
  var canvas = document.getElementById('c');
  var ctx = canvas.getContext('2d');

  canvas.toBlob(t.step_func_done(blob => {
    assert_greater_than(blob.size, 0);
  }));

}, "Example asynchronous test");
```

Promise-based APIs would use `test_type: promise`, for instance:
```yaml
- name: 2d.sample.promise-test
  desc: Example promise test
  canvas_types: ['OffscreenCanvas']
  test_type: promise
  code: |
    const blob = await canvas.convertToBlob();
    assert_greater_than(blob.size, 0);
```
```JavaScript
promise_test(async t => {

  var canvas = new OffscreenCanvas(100, 50);
  var ctx = canvas.getContext('2d');

  const blob = await canvas.convertToBlob();
  assert_greater_than(blob.size, 0);

}, "Example promise test");
```

To maintain compatibility with old tests (until they are updated), the test
generator will use a legacy test harness if the `test_type` is omitted. This
test harness is a quirky hybrid between `sync` and `async` tests. These tests
are implemented with an `async_test()` fixture, but the generator automatically
invokes `t.done()` after the body, making is behave like a synchronous test. To
implement actual async test, the test body must call `deferTest();` and manually
call `t.done()` to finish the test. Newer test should prefer specifying
`test_type` and using standard `testharness.js` APIs instead of these
canvas-specific ones.


## Reftests
The code generator can also be used to generate [reftests](
https://web-platform-tests.org/writing-tests/reftests.html). These tests are
comprised of a test HTML page and a reference HTML page. Both are rendered by
the test runner and the results are compared pixel by pixel. These tests do not
use `testharness.js` and thus cannot use assertions.

### Writing reftests

To write a reftest, use a `reference:` key in the YAML config:
```yaml
- name: 2d.sample.reftest
  desc: Example reftest
  code: |
    ctx.scale(2, 2);
    ctx.fillRect(5, 5, 15, 15);
  reference: |
    ctx.fillRect(10, 10, 30, 30);
```

This will produce a test file `2d.sample.reftest.html` and a reference file
`2d.sample.reftest-expected.html`, for HTMLCanvasElement and main thread/worker
OffscreenCanvas.

### Reftest fuzzy matching

By default, the test will fail if the test runner sees any rendering difference
between the test and reference page. If some difference is expected,
fuzzy-matching can be used by using the `fuzzy:` config, as in:
```yaml
- name: 2d.sample.reftest.fuzzy
  desc: Example reftest using fuzzy matching
  fuzzy: maxDifference=0-1; totalPixels=0-100
  code: |
    ctx.fillStyle = 'rgba(0, 255, 0, 0.5)';
    ctx.fillRect(5, 5, 10, 10);
  reference: |
    ctx.fillStyle = 'rgba(128, 255, 128, 1)';  // Should it be 127 or 128?
    ctx.fillRect(5, 5, 10, 10);
```

### Types of reference files
The code generator supports three types of reference files that can be used to
validated the test results.

- `reference:` generates a reference file using JavaScript code writing to a
canvas, similarly to what the `code:` block of the test does. For instance:
```yaml
- name: 2d.sample.reftest.reference
  desc: Reftest comparing one canvas drawing with another.
  code: |
    ctx.scale(2, 2);
    ctx.fillRect(5, 5, 15, 15);
  reference: |
    ctx.fillRect(10, 10, 30, 30);
```

- `html_reference:` can be useful to compare canvas rendering against HTML+CSS, or
against SVG. For instance:
```yaml
- name: 2d.sample.reftest.html_reference
  desc: Example reftest using an html_reference
  fuzzy: maxDifference=0-1; totalPixels=0-24
  code: |
    ctx.filter = 'blur(5px)';
    ctx.fillRect(5, 5, 10, 10);
  html_reference: |
    <svg xmlns="https://www.w3.org/2000/svg"
         width="{{ size[0] }}" height="{{ size[1] }}">
      <filter id="filter" x="-100%" y="-100%" width="300%" height="300%">
        <feGaussianBlur stdDeviation="5" />
      </filter>

      <g filter="url(#filter)">
        <rect x="5" y="5" width="10" height="10"></rect>
      </g>
    </svg>
```

- `cairo_reference:` produces a reftest using a reference image generated from
Python code using the `pycairo` library. The variable `cr` provides a
`cairo.Context` instance that can be used to draw the reference image. For
instance:
```yaml
- name: 2d.sample.reftest.cairo_reference
  desc: Example reftest using a cairo_reference
  code: |
    ctx.fillStyle = 'rgb(255, 128, 64)';
    ctx.fillRect(5, 5, 50, 30);
    ctx.globalCompositeOperation = 'color-dodge';
    ctx.fillStyle = 'rgb(128, 200, 128)';
    ctx.fillRect(15, 15, 50, 30);
  cairo_reference: |
    cr.set_source_rgb(255/255, 128/255, 64/255)
    cr.rectangle(5, 5, 50, 30)
    cr.fill()
    cr.set_operator(cairo.OPERATOR_COLOR_DODGE)
    cr.set_source_rgb(128/255, 200/255, 128/255)
    cr.rectangle(15, 15, 50, 30)
    cr.fill()
```

- `img_reference:` produces a reftest using a pre-generated image file. This can
be useful for trivial images (e.g. a solid color), but avoid using this for
non-trivial images as it's important to be able to know how these images are
generated, so we could inspect or modify them later if needed. Example:
```yaml
- name: 2d.sample.reftest.img_reference
  desc: Example reftest using an img_reference
  size: [100, 50]
  code: |
    ctx.fillStyle = 'rgb(0, 255, 0)';
    ctx.fillRect(0, 0, {{ size[0] }}, {{ size[1] }});
  img_reference: /images/green-100x50.png
```

### Promise reftests
As explained [in the WPT documentation](
https://web-platform-tests.org/writing-tests/reftests.html), the test runner
screenshots reftests after the page is loaded and pending paints are completed.
To support asynchronous calls or promises, we must let the test runner know when
the test is done. This can be done by using `test_type: promise`, for instance:

```yaml
- name: 2d.sample.reftest.promise
  desc: Example of a reftest using promises
  test_type: promise
  code: |
    ctx.beginLayer();
    ctx.fillRect(5, 5, 10, 10);
    // Checks that layers survive frame boundaries.
    await new Promise(resolve => requestAnimationFrame(resolve));
    ctx.endLayer();
  reference:
    ctx.fillRect(5, 5, 10, 10);
```

## Test variants

Test parameterization is a very useful tool, allowing a test to be exercised on
a number of different inputs with minimal boilerplate. The test generator
supports this via the `variants:` parameter. Let's begin by showing a motivating
example. Say you would like to compare canvas rendering with SVG. You might want
to implement these two tests:

```yaml
- name: 2d.compare-canvas-and-svg.draws-red
  color: 'red'
  code: |
    ctx.fillStyle = '{{ color }}';
    ctx.fillRect(0, 0, 10, 10);
  html_reference: |
    <svg xmlns="https://www.w3.org/2000/svg"
          width="{{ size[0] }}" height="{{ size[1] }}">
      <rect x="0" y="0" width="10" height="10" fill="{{ color }}"/>
    </svg>

- name: 2d.compare-canvas-and-svg.draws-green
  color: 'green'
  code: |
    ctx.fillStyle = '{{ color }}';
    ctx.fillRect(0, 0, 10, 10);
  html_reference: |
    <svg xmlns="https://www.w3.org/2000/svg"
          width="{{ size[0] }}" height="{{ size[1] }}">
      <rect x="0" y="0" width="10" height="10" fill="{{ color }}"/>
    </svg>
```

This works, but there's a lot of duplication and it would not scale for testing
with a larger number of inputs. You could cut down on the duplication by using
YAML `&` anchors and `*` aliases, for instance:
```yaml
- name: 2d.compare-canvas-and-svg.draws-red
  fill_style: 'red'
  code: &draw-rect-with-canvas |
    ctx.fillStyle = '{{ fill_style }}';
    ctx.fillRect(0, 0, 10, 10);
  html_reference: &draw-rect-with-svg |
    <svg xmlns="https://www.w3.org/2000/svg"
          width="{{ size[0] }}" height="{{ size[1] }}">
      <rect x="0" y="0" width="10" height="10" fill="{{ fill_style }}"/>
    </svg>

- name: 2d.compare-canvas-and-svg.draws-green
  fill_style: 'green'
  code: *draw-rect-with-canvas
  html_reference: *draw-rect-with-svg
```

This however adds indirections making the test hard to read. Test must still be
duplicated, but they now mostly contain boilerplate and this would still not
scale for a larger number of test inputs. With variants, you could write the
test like this:

```yaml
- name: 2d.compare-canvas-and-svg
  code: |
    ctx.fillStyle = '{{ fill_style }}';
    ctx.fillRect(0, 0, 10, 10);
  html_reference: |
    <svg xmlns="https://www.w3.org/2000/svg"
          width="{{ size[0] }}" height="{{ size[1] }}">
      <rect x="0" y="0" width="10" height="10" fill="{{ fill_style }}"/>
    </svg>
  variants:
  - draws-red: {fill_style: red}
    draws-green: {fill_style: green}
```

Now, this has minimal boilerplate and easily scales to many more test inputs.

Variant are defined as a list of dictionaries of dictionaries. In the above
examples, the `-` before `draws-red:` denotes a list item, which is a dictionary
with two keys: `draws-red` and `draws-green`. That dictionary defines a variant
dimension. A test file will be generated for each items in this dictionary. The
`draws-red` and `draws-green` strings are the variant names, which are appended
to the generated test name. The value associated with these keys (for instance
`{fill_style: red}`) are parameters that get merged over the base test
parameters for that test instance.

If the `variants:` list has more than one dictionary, the test generator will
generate tests with the cross product of all variant dimensions. For instance,
this config:

```yaml
  - name: 2d.grid
    code: ctx.{{ variant_names[0] }} = '{{ color }}';
    variants:
    - fillStyle:
      shadowColor:
    - red: {color: '#F00'}
      blue: {color: '#00F'}
```
Will generate:
- Test name: `2d.grid.fillStyle.red`, with code: `ctx.fillStyle = '#F00';`
- Test name: `2d.grid.fillStyle.blue`, with code: `ctx.fillStyle = '#00F';`
- Test name: `2d.grid.shadowColor.red`, with code: `ctx.shadowColor = '#F00';`
- Test name: `2d.grid.shadowColor.blue`, with code: `ctx.shadowColor = '#00F';`

This last example uses the variable `{{ variant_names[0] }}` to avoid having to
write `fillStyle` and `shadowColor` twice, once in the variant name and once on
the variant params. `variant_names` is a special variable which holds a list of
the variant names for that particular test instance. When generating the test
`2d.grid.fillStyle.red` for instance, `variant_names` would be set to
`['fillStyle', 'red']`.

## Grid tests
Variants are really useful to easily validate many test inputs, but this can
lead to a large number of test files being generated. For instance, the 2d
canvas compositing pipeline is affected by these states (and more):
 - Drawing with `fillRect`, `drawImage`, `CanvasPattern`.
 - globalAlpha to `1.0` or non `1.0`.
 - All 26 `globalCompositeOperation`
 - With shadow enabled or disabled.
 - Using a filter or not.
 - With transforms or not.
 - With an opaque or transparent canvas.
 - For HTMLCanvasElement, main thread OffscreenCanvas and worker
   OffscreenCanvas.

In practice, we have found bugs in the canvas what only happen for very specific
combinations of states, like, using a specific `globalCompositeOperation`, with
transforms and shadows enabled, but only when `drawImage` is used. We might want
be thorough and test as many combinations of these states as we can, but testing
all permutations would require $`3*2*26*2*2*2*2*3 = 7488`$ tests. Putting all of
these in different files would be unmanageable. Comes variant grids to the
rescue! If we generated test files each testing all composite operations in a
grid, we'd only need a much more reasonable $`3*2*2*2*2*2*3 = 288`$ files.

To generate a variant grid, use the `variants_layout:` config. This has to be a
list of the same lengths as the `variants:` list (as long as there are variant
dimensions). For each dimension, `variants_layout:` must hold either the string
`multi_files` or `single_file`. Variant dimensions marked as `single_file` will
be expanded in the same file. For instance:
```yaml
- name: grid-example
  variants:
  - A1:
    A2:
  - B1:
    B2:
  - C1:
    C2:
  - D1:
    D2:
  - E1:
    E2:
  variants_layout:
    - single_file
    - multi_files
    - single_file
    - multi_files
    - single_file
```

Because this test has 2 `multi_files` dimension with two variants each, 4 files
would be generated:
  - grid-example.B1.D1
  - grid-example.B1.D2
  - grid-example.B2.D1
  - grid-example.B2.D2

Then, the 3 `single_file` dimensions would produce $`2*2*2 = 8`$ tests in each
of these files. For JavaScript tests, each of these test would be generated in
sequence, each with their own `test()`, `async_test()` or `promise_test()`
fixture. Reftest on the other hand would produce a 2x2x2 grid, as follows:
```
   A1.C1.E1     A2.C1.E1
   A1.C2.E1     A2.C2.E1
   A1.C1.E2     A2.C1.E2
   A1.C2.E2     A2.C2.E2
```
