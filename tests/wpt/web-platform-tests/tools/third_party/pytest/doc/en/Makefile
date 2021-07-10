# Minimal makefile for Sphinx documentation
#

# You can set these variables from the command line, and also
# from the environment for the first two.
SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
%: Makefile
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)


REGENDOC_ARGS := \
	--normalize "/[ \t]+\n/\n/" \
	--normalize "~\$$REGENDOC_TMPDIR~/home/sweet/project~" \
	--normalize "~/path/to/example~/home/sweet/project~" \
	--normalize "/in \d.\d\ds/in 0.12s/" \
	--normalize "@/tmp/pytest-of-.*/pytest-\d+@PYTEST_TMPDIR@" \
	--normalize "@pytest-(\d+)\\.[^ ,]+@pytest-\1.x.y@" \
	--normalize "@py-(\d+)\\.[^ ,]+@py-\1.x.y@" \
	--normalize "@pluggy-(\d+)\\.[.\d,]+@pluggy-\1.x.y@" \
	--normalize "@hypothesis-(\d+)\\.[.\d,]+@hypothesis-\1.x.y@" \
	--normalize "@Python (\d+)\\.[^ ,]+@Python \1.x.y@"

regen: REGENDOC_FILES:=*.rst */*.rst
regen:
	PYTHONDONTWRITEBYTECODE=1 PYTEST_ADDOPTS="-pno:hypothesis -Wignore::pytest.PytestUnknownMarkWarning" COLUMNS=76 regendoc --update ${REGENDOC_FILES} ${REGENDOC_ARGS}

.PHONY: regen
