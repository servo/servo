.PHONY: clean clean-build clean-pyc clean-test lint test

clean: clean-build clean-pyc clean-test ## remove all build, test, coverage and Python artifacts

clean-build: ## remove build artifacts
	rm -fr build/
	rm -fr dist/
	rm -fr .eggs/
	find . -name '*.egg-info' -exec rm -fr {} +
	find . -name '*.egg' -exec rm -f {} +

clean-pyc: ## remove Python file artifacts
	find . -name '*.pyc' -exec rm -f {} +
	find . -name '*.pyo' -exec rm -f {} +
	find . -name '*~' -exec rm -f {} +
	find . -name '__pycache__' -exec rm -fr {} +

clean-test: ## remove test and coverage artifacts
	rm -fr .tox/
	rm -f .coverage
	rm -fr htmlcov/

lint:
# CI env-var is set by GitHub actions
ifdef CI
	python -m pre_commit run --all-files --show-diff-on-failure
else
	python -m pre_commit run --all-files
endif
	python -m mypy pytest_asyncio --show-error-codes

test:
	coverage run -m pytest tests
	coverage xml
	coverage report

install:
	pip install -U pre-commit
	pre-commit install
