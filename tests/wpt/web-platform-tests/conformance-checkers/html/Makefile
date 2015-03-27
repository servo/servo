PYTHON=python
PYTHONFLAGS=
META_EXTENSIONS=http://help.whatwg.org/extensions/meta-name/
CURL=curl
CURLFLAGS=
PYTHON=python
PYTHONFLAGS=

.PHONY: ins-del-datetime .FORCE

all: ins-del-datetime tools/meta-extensions elements/meta/names-registered-isvalid.html

ins-del-datetime: tools/ins-del-datetime.py
	$(PYTHON) $(PYTHONFLAGS) $<

tools/meta-extensions: .FORCE
	$(CURL) $(CURLFLAGS) $(META_EXTENSIONS) > $@

elements/meta/names-registered-isvalid.html: .FORCE
	$(PYTHON) $(PYTHONFLAGS) tools/meta-name.py
