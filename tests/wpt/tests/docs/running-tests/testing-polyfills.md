# Testing polyfills

## Preparing the polyfill

The polyfill script-injection feature currently only supports scripts which
are immediately invoked. The script must be prepared as a single file whose
contents will be inlined into a script tag served as part of every test page.

If your polyfill is only available as an asynchronous module with dependent
scripts, you can use a tool such as
[microbundle](https://github.com/developit/microbundle) to repackage it as a
single synchronous script file, e.g.:

```bash
microbundle -f iife -i polyfill/src/main.js -o polyfill.js
```

## Running the tests

Follow the steps for [Running Tests from the Local System](from-local-system) to
set up your test environment. When running tests via the browser or via the
command line, add the `--inject-script=polyfill.js` to either command, e.g.

Via the browser:

```bash
./wpt serve --inject-script=polyfill.js
```

Then visit http://web-platform.test:8000/ or https://web-platform.test:8443/ to
run the tests in your browser.

Via the command line:

```bash
./wpt run --inject-script=polyfill.js [browsername] [tests]
```

## Limitations

Polyfill scripts are injected to an inline script tag which removes itself from
the DOM after executing. This is done by modifying the server response for
documents with a `text/html` MIME type  to insert the following before the first tag in
the served response:

```html
<script>
// <-- The polyfill file is inlined here
// Remove the injected script tag from the DOM.
document.currentScript.remove();
```

This approach has a couple limitations:
* This requires that the polyfill is self-contained and executes
synchronously in a single inline script. See [Preparing the
polyfill](#preparing-the-polyfill) for suggestions on transforming polyfills to
run in that way.
* Does not inject into python handlers which write directly to the output
  stream.
* Does not inject into the worker context of `.any.js` tests.

### Observability

The script tag is removed from the DOM before any other script has run, and runs
from an inline script. As such, it should not affect mutation observers on the
same page or resource timing APIs, as it is not a separate resource. The polyfill
may be observable by a mutation observer added by a parent frame before load.
