# Custom Products

External browser products can be registered with wptrunner using entry points. Once installed, these products are available to the `./wpt run` command.

## Installation

Custom products do not support automatic browser installation. Install the browser
manually before running tests.

Which command-line arguments are required (such as `--binary` or
`--webdriver-binary`) depends on the product plugin. Consult the plugin's
documentation.

## Usage

First, install the custom product plugin into the same Python that `./wpt` uses:

```bash
python3 -m pip install wptrunner-mybrowser
```

Then run tests using the product name:

```bash
./wpt run mybrowser test.html
```

Some products require a browser binary path:

```bash
./wpt run mybrowser --binary=/path/to/mybrowser test.html
```

Some products require a WebDriver binary path:

```bash
./wpt run mybrowser --webdriver-binary=/path/to/mydriver test.html
```

Additional arguments can be passed to the browser using `--binary-arg`:

```bash
./wpt run mybrowser --binary=/path/to/mybrowser --binary-arg=--headless test.html
```

## Troubleshooting

If you see "Product mybrowser not found":

* The plugin may not be installed: `python3 -m pip install wptrunner-mybrowser`
* Check that the plugin is registered correctly (run `python3 -m pip show wptrunner-mybrowser`)

For information on creating custom product plugins, see the
[wptrunner plugin documentation](../../tools/wptrunner/docs/plugins.rst).
