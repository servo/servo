wget https://mxr.mozilla.org/mozilla-central/source/dom/bindings/parser/WebIDL.py?raw=1 -O WebIDL.py
patch < abstract.patch
patch < debug.patch
patch < legacy-unenumerable-named-properties.patch

wget -r -np -l1 -A py -nd https://dxr.mozilla.org/mozilla-central/source/dom/bindings/parser/tests -P tests

# TODO: update test files from https://dxr.mozilla.org/mozilla-central/source/dom/bindings/parser/tests
