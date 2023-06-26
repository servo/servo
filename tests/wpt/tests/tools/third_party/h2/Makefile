.PHONY: publish test

publish:
	rm -rf dist/
	python setup.py sdist bdist_wheel
	twine upload -s dist/*

test:
	py.test -n 4 --cov h2 test/
