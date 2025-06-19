wget https://hg.mozilla.org/mozilla-central/raw-file/tip/dom/bindings/parser/WebIDL.py -O WebIDL.py
patch < abstract.patch
patch < debug.patch
patch < callback-location.patch
patch < union-typedef.patch
patch < inline.patch
patch < like-as-iterable.patch
patch < builtin-array.patch
patch < array-type.patch
patch < transferable.patch
patch < dom-stubs.patch

wget https://hg.mozilla.org/mozilla-central/archive/tip.zip/dom/bindings/parser/tests/ -O tests.zip
rm -r tests
mkdir tests
unzip -d tests -j tests.zip
rm tests.zip WebIDL.py.orig
