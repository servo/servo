// Note that the function source text is intentionally different from e.g.
// ../alpha/code-cache.js to avoid caching Functions between different sets
// of tests.
parent.promise = (new Function('x', 'return import(x)'))('../../imports-a.js');
