wget https://mxr.mozilla.org/mozilla-central/source/dom/bindings/parser/WebIDL.py?raw=1 -O WebIDL.py
patch < abstract.patch
patch < debug.patch
patch < pref-main-thread.patch

wget https://hg.mozilla.org/mozilla-central/archive/tip.tar.gz/dom/bindings/parser/tests/ -O tests.tar.gz
rm -r tests
mkdir tests
tar xvpf tests.tar.gz  -C tests --strip-components=5
rm tests.tar.gz WebIDL.py.orig
