wget https://hg.mozilla.org/mozilla-central/raw-file/tip/dom/bindings/parser/WebIDL.py -O WebIDL.py
patch < abstract.patch
patch < debug.patch
patch < pref-main-thread.patch
patch < callback-location.patch
patch < union-typedef.patch
patch < inline.patch

wget https://hg.mozilla.org/mozilla-central/archive/tip.tar.gz/dom/bindings/parser/tests/ -O tests.tar.gz
rm -r tests
mkdir tests
tar xvpf tests.tar.gz  -C tests --strip-components=5
rm tests.tar.gz WebIDL.py.orig
