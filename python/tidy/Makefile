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
	$(VENV)/bin/python -m unittest discover -s servo_tidy_tests -v


.PHONY: clean
clean:
	find . -name "*.pyc" -type f -delete
	rm -rf $(VENV)
