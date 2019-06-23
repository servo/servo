# Test Templates

This page contains templates for creating tests. The template syntax
is compatible with several popular editors including TextMate, Sublime
Text, and emacs' YASnippet mode.

Templates for filenames are also given. In this case `{}` is used to
delimit text to be replaced and `#` represents a digit.

## Reftests

### Test

<!--
  Syntax highlighting cannot be enabled for the following template because it
  contains invalid CSS.
-->

```
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

### Reference:

<!--
  Syntax highlighting cannot be enabled for the following template because it
  contains invalid CSS.
-->

```
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

## testharness.js tests

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
