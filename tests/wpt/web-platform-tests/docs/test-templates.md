This page contains templates for creating tests. The template syntax
is compatible with several popular editors including TextMate, Sublime
Text, and emacs' YASnippet mode.

Each template is given in two forms, one minimal and one including
[extra metadata](css-metadata.html). Usually the metadata is required
by CSS tests and optional for other tests.

Templates for filenames are also given. In this case `{}` is used to
delimit text to be replaced and `#` represents a digit.

## Reftests

### Minimal Reftest

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test title}</title>
<link rel="match" href="${2:URL of match}">
<style>
    ${3:Test CSS}
</style>
<body>
    ${4:Test content}
</body>
```

Filename: `{test-topic}-###.html`

### Reftest Including Metadata

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test area}: ${2:Scope of test}</title>
<link rel="author" title="${3:Author's name}" href="${4:Contact link}">
<link rel="help" href="${5:Link to tested section}">
<link rel="match" href="${6:URL of match}">
<meta name="flags" content="${7:Requirement flags}">
<meta name="assert" content="${8:Description of what you're trying to test}">
<style>
    ${9:Test CSS}
</style>
<body>
    ${10:Test content}
</body>
```

Filename: `{test-topic}-###.html`

### Minimal Reftest Reference:

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Reference title}</title>
<style>
    ${2:Reference CSS}
</style>
<body>
    ${3:Reference content}
</body>
```

Filename: `{description}.html` or `{test-topic}-###-ref.html`

### Reference Including Metadata

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Reference title}</title>
<link rel="author" title="${2:Author's name}" href="${3:Contact link}">
<style>
    ${4:Reference CSS}
</style>
<body>
    ${5:Reference content}
</body>
```

Filename: `{description}.html` or `{test-topic}-###-ref.html`

## testharness.js tests

### Minimal Script Test

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test title}</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
${2:Test body}
</script>
```

Filename: `{test-topic}-###.html`

### Script Test With Metadata

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test title}</title>
<link rel="author" title="${2:Author's name}" href="${3:Contact link}">
<link rel="help" href="${4:Link to tested section}">
<meta name="flags" content="${5:Requirement flags}">
<meta name="assert" content="${6:Description of what you're trying to test}">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
${7:Test body}
</script>
```

Filename: `{test-topic}-###.html`

### Manual Test

``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test title}</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
setup({explicit_timeout: true});
${2:Test body}
</script>
```

Filename: `{test-topic}-###-manual.html`
