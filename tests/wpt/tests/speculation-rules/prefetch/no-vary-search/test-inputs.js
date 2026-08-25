// Test inputs:
// - description: a description of the test.
// - noVarySearch: No-Vary-Search header value for the response.
// - prefetchQuery: added to query part of prefetch-executor when prefetching
// - navigateQuery: added to query part of prefetch-executor when navigating
// - shouldUse: if the test case expects the prefetched entry to be used or not.
const test_inputs = [
   {description:"Use prefetched response as query parameter b has the same value.",
    noVarySearch: 'params=("a")',
    prefetchQuery: "a=2&b=3",
    navigateQuery: "b=3",
    shouldUse: true},

   {description:"Don't use prefetched response as query parameter b has different value.",
    noVarySearch: 'params("a")',
    prefetchQuery: "a=2&b=3",
    navigateQuery: "b=2",
    shouldUse: false},

   {description:"Use prefetched response as the URLs do not vary by a and b.",
    noVarySearch: 'params=("a" "b")',
    prefetchQuery: "a=2&b=3",
    navigateQuery: "b=2",
    shouldUse: true},

   {description:"Use prefetched response as the URLs do not vary on any query parameters.",
    noVarySearch: "params",
    prefetchQuery: "a=2&b=3",
    navigateQuery: "b=4&c=5",
    shouldUse: true},

   {description:"Use prefetched response as the URLs do not vary on any query parameters.",
    noVarySearch: "params",
    prefetchQuery: "",
    navigateQuery: "b=4&c=5",
    shouldUse: true},

   {description:"Don't use prefetched response as the URLs have different value for c.",
    noVarySearch: "key-order",
    prefetchQuery: "c=4&b=3&a=2",
    navigateQuery: "a=2&c=5&b=3",
    shouldUse: false},

   {description:"Don't use prefetched response as the URLs have the values in different order for a.",
    noVarySearch: "key-order",
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5&b=3",
    navigateQuery: "d=6&a=4&b=5&b=3&c=5&a=3",
    shouldUse: false},

   {description:"Use prefetched response as the URLs have the same values for a.",
    noVarySearch: "key-order",
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5&b=3",
    navigateQuery: "d=6&a=3&b=5&b=3&c=5&a=4",
    shouldUse: true},

   {description:"Use prefetched response as the URLs have the same values for a.",
    noVarySearch: "key-order=?1",
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5&b=3",
    navigateQuery: "d=6&a=3&b=5&b=3&c=5&a=4",
    shouldUse: true},

   {description:"Don't use prefetched response as key-order is set to false and the URLs are not identical.",
    noVarySearch: "key-order=?0",
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5&b=3",
    navigateQuery: "d=6&a=3&b=5&b=3&c=5&a=4",
    shouldUse: false},

   {description:"Use prefetched response as query parameter c can be ignored.",
    noVarySearch: 'params=("c")',
    prefetchQuery: "a=2&b=2&c=5",
    navigateQuery: "a=2&c=3&b=2",
    shouldUse: true},

   {description:"Use prefetched response as query parameter a can be ignored.",
    noVarySearch: 'params=("a")',
    prefetchQuery: "a=2",
    navigateQuery: "",
    shouldUse: true},

   {description:"Use prefetched response as query parameter a can be ignored.",
    noVarySearch: 'params=("a")',
    prefetchQuery: "",
    navigateQuery: "a=2",
    shouldUse: true},

   {description:"Use prefetched response as all query parameters except c can be ignored.",
    noVarySearch: 'params, except=("c")',
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=3",
    shouldUse: true},

   {description:"Use prefetched response as all query parameters except c can be ignored." +
                " Only the last except matters.",
    noVarySearch: 'params, except=("b"), except=("c")',
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=3",
    shouldUse: true},

   {description:"Don't use prefetched response as even though all query parameters" +
                " except c can be ignored, c has different value.",
    noVarySearch: 'params, except=("c")',
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=5",
    shouldUse: false},

   {description:"Use prefetched response as even though all query parameters" +
                " except c and d can be ignored, c value matches and d value matches.",
    noVarySearch: 'params, except=("c" "d")',
    prefetchQuery: "b=5&a=3&d=6&c=5",
    navigateQuery: "d=6&a=1&b=2&c=5",
    shouldUse: true},

   {description:"Use prefetched response as even though all query parameters except" +
                " c and d can be ignored, c value matches and d value matches." +
                " Some query parameters to be ignored appear multiple times in the query.",
    noVarySearch: 'params, except=("c" "d")',
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5",
    navigateQuery: "d=6&a=1&a=2&b=2&b=3&c=5",
    shouldUse: true},

   {description:"Use prefetched response as all query parameters except c can be ignored." +
                " Allow extension via parameters.",
    noVarySearch: 'params, except=("c";unknown)',
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=3",
    shouldUse: true},

   {description:"Use prefetched response as query parameter c can be ignored." +
                " Allow extension via parameters.",
    noVarySearch: 'params=("c";unknown)',
    prefetchQuery: "a=2&b=2&c=5",
    navigateQuery: "a=2&c=3&b=2",
    shouldUse: true},

   {description:"Use prefetched response as the URLs have the values in different order for a." +
                " Allow extension via parameters.",
    noVarySearch: "key-order;unknown",
    prefetchQuery: "b=5&a=3&a=4&d=6&c=5&b=3",
    navigateQuery: "d=6&a=3&b=5&b=3&c=5&a=4",
    shouldUse: true},

   {description:"Use prefetched response as the URLs do not vary on any query parameters." +
                " Allow extension via parameters.",
    noVarySearch: "params;unknown",
    prefetchQuery: "",
    navigateQuery: "b=4&c=5",
    shouldUse: true},

   {description:"Use prefetched response as all query parameters except c can be ignored." +
                " Allow extension via parameters.",
    noVarySearch: 'params;unknown, except=("c");unknown',
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=3",
    shouldUse: true},

   {description:"Don't use the prefetched URL. Empty No-Vary-Search means default URL variance." +
                " The prefetched and the navigated URLs have to be the same.",
    noVarySearch: "",
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "a=1&b=2&c=3",
    shouldUse: false},

   {description:"Use the prefetched URL. Empty No-Vary-Search means default URL variance." +
                " The prefetched and the navigated URLs have to be the same.",
    noVarySearch: "",
    prefetchQuery: "b=5&a=3&d=6&c=3",
    navigateQuery: "b=5&a=3&d=6&c=3",
    shouldUse: true},

   {description:"Use the prefetched URL. Empty No-Vary-Search means default URL variance." +
                " The prefetched and the navigated URLs have to be the same.",
    noVarySearch: "",
    prefetchQuery: "",
    navigateQuery: "",
    shouldUse: true},

   {description:"Use the prefetched URL. Non-ASCII key - 2 UTF-8 code units." +
                " Don't vary the response on the non-ASCII key.",
    noVarySearch: 'params=("%C2%A2")',
    prefetchQuery: "¢=3",
    navigateQuery: "¢=4",
    shouldUse: true},

    {description:"Use the prefetched URL. Non-ASCII key - 2 UTF-8 code units." +
                 " Don't vary the response on the non-ASCII key.",
    noVarySearch: 'params=("%C2%A2")',
    prefetchQuery: "a=2&¢=3",
    navigateQuery: "¢=4&a=2",
    shouldUse: true},

    {description:"Don't use the prefetched URL. Non-ASCII key - 2 UTF-8 code units." +
                 " Vary the response on the non-ASCII key.",
    noVarySearch: 'params, except=("%C2%A2")',
    prefetchQuery: "¢=3",
    navigateQuery: "¢=4",
    shouldUse: false},

    {description:"Use the prefetched URL. Non-ASCII key - 2 UTF-8 code units." +
                 " Vary the response on the non-ASCII key.",
    noVarySearch: 'params, except=("%C2%A2")',
    prefetchQuery: "¢=3&a=4",
    navigateQuery: "a=5&¢=3",
    shouldUse: true},
];
