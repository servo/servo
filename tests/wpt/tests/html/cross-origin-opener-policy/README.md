This directory as well as `../cross-origin-embedder-policy/` contains tests for `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy`. Some light background reading:

* [COOP and COEP explained](https://docs.google.com/document/d/1zDlfvfTJ_9e8Jdc8ehuV4zMEu9ySMCiTGMS9y0GU92k/edit)
* [COOP processing model](https://gist.github.com/annevk/6f2dd8c79c77123f39797f6bdac43f3e) (also defines interaction with COEP)
* [COEP processing model](https://mikewest.github.io/corpp/)
* [Open COOP issues](https://github.com/whatwg/html/labels/topic%3A%20cross-origin-opener-policy)
* [Open COEP issues](https://github.com/whatwg/html/labels/topic%3A%20cross-origin-embedder-policy)

Notes:

* Top-level navigation to a `data:` URL does not work in Chrome and Firefox and is therefore not tested. (This should probably be standardized.)
