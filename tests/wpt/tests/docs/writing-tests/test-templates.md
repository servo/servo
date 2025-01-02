# Test Templates

This page contains templates for creating tests. The template syntax
is compatible with several popular editors including TextMate, Sublime
Text, and emacs' YASnippet mode.

Templates for filenames are also given. In this case `{}` is used to
delimit text to be replaced and `#` represents a digit.

## Reftests

### HTML test

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

### HTML reference

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

### SVG test

``` xml
<svg xmlns="http://www.w3.org/2000/svg" xmlns:h="http://www.w3.org/1999/xhtml">
  <title>${1:Test title}</title>
  <metadata>
    <h:link rel="help" href="${2:Specification link}"/>
    <h:link rel="match" href="${3:URL of match}"/>
  </metadata>
  ${4:Test body}
</svg>
```

Filename: `{test-topic}-###.svg`

### SVG reference

``` xml
<svg xmlns="http://www.w3.org/2000/svg">
  <title>${1:Reference title}</title>
  ${2:Reference content}
</svg>
```

Filename: `{description}.svg` or `{test-topic}-###-ref.svg`

## testharness.js tests

### HTML

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

### HTML with [testdriver automation](testdriver)
``` html
<!DOCTYPE html>
<meta charset="utf-8">
<title>${1:Test title}</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/resources/testdriver.js"></script>
<script src="/resources/testdriver-vendor.js"></script>

<script>
${2:Test body}
</script>
```

Filename: `{test-topic}-###.html`

### SVG

``` xml
<svg xmlns="http://www.w3.org/2000/svg" xmlns:h="http://www.w3.org/1999/xhtml">
  <title>${1:Test title}</title>
  <metadata>
    <h:link rel="help" href="${2:Specification link}"/>
  </metadata>
  <h:script src="/resources/testharness.js"/>
  <h:script src="/resources/testharnessreport.js"/>
  <script><![CDATA[
  ${4:Test body}
  ]]></script>
</svg>
```

Filename: `{test-topic}-###.svg`

### Manual Test

#### HTML

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

#### SVG

``` xml
<svg xmlns="http://www.w3.org/2000/svg" xmlns:h="http://www.w3.org/1999/xhtml">
  <title>${1:Test title}</title>
  <metadata>
    <h:link rel="help" href="${2:Specification link}"/>
  </metadata>
  <h:script src="/resources/testharness.js"/>
  <h:script src="/resources/testharnessreport.js"/>
  <script><![CDATA[
  setup({explicit_timeout: true});
  ${4:Test body}
  ]]></script>
</svg>
```

Filename: `{test-topic}-###-manual.svg`
