// META: script=/loading/early-hints/resources/early-hints-helpers.sub.js

// Test matrix: Early Hints preloads with various as/crossorigin combinations.
const testCases = [
  {as_attr: 'script'},
  {as_attr: 'script', crossorigin_attr: 'anonymous'},
  {as_attr: 'script', crossorigin_attr: 'use-credentials'},
  {as_attr: 'style'},
  {as_attr: 'style',  crossorigin_attr: 'anonymous'},
  {as_attr: 'style',  crossorigin_attr: 'use-credentials'},
  {as_attr: 'fetch'},
  {as_attr: 'fetch',  crossorigin_attr: 'anonymous'},
  {as_attr: 'fetch',  crossorigin_attr: 'use-credentials'},
];

const resourceFiles = {
  'script': 'empty.js',
  'style':  'empty.css',
  'fetch':  'empty.json',
};

test(() => {
    const preloads = testCases.map(tc => {
        const entry = {
            url: resourceFiles[tc.as_attr] + '?' + tc.as_attr +
                 '-' + (tc.crossorigin_attr || 'none') + '-' + Date.now(),
            as_attr: tc.as_attr,
        };
        if ('crossorigin_attr' in tc) {
            entry.crossorigin_attr = tc.crossorigin_attr;
        }
        return entry;
    });
    navigateToTestWithEarlyHints(
        "resources/speculation-measurement-early-hints.html",
        preloads);
});
