wget https://mxr.mozilla.org/mozilla-central/source/dom/bindings/parser/WebIDL.py?raw=1 -O WebIDL.py
patch < abstract.patch
patch < legacy-unenumerable-named-properties.patch

# TODO: update test files from https://dxr.mozilla.org/mozilla-central/source/dom/bindings/parser/tests
