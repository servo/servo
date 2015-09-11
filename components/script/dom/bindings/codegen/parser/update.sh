wget https://mxr.mozilla.org/mozilla-central/source/dom/bindings/parser/WebIDL.py?raw=1 -O WebIDL.py
patch < external.patch
patch < module.patch
patch < abstract.patch
