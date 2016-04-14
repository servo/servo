PYTHON := python
VENV := env-$(PYTHON)

# for travis

$(VENV)/bin/python:
	[ -d $(VENV) ] || $(PYTHON) -m virtualenv $(VENV) || virtualenv $(VENV)
	$(VENV)/bin/pip install --upgrade setuptools
	$(VENV)/bin/python setup.py develop


.PHONY: dev-env
dev-env: $(VENV)/bin/python


# for testing
.PHONY: test
test: dev-env
	$(VENV)/bin/pip install --upgrade funcsigs
	$(VENV)/bin/pip install --upgrade mock
	$(VENV)/bin/python -m unittest discover -s tests -v


.PHONY: clean
clean:
	find . -name "*.pyc" -type f -delete


