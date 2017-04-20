##############################################################################
# W3C Test Suite Generator                                                   #
##############################################################################

## Variables

DESTINATION_TYPES=$(shell perl generate.pl -v DESTINATION_TYPES)
SHELL_TYPES=$(shell perl generate.pl -v SHELL_TYPES)
TEST_TYPES=$(shell perl generate.pl -v TEST_TYPES)
SOURCE=$(shell ls *.xml)
STYLESHEETS=dist/style/tng.css $(shell perl -e 'foreach $$a (qw($(DESTINATION_TYPES))) { print "dist/style/$$a-full.css\ndist/style/$$a-shell.css\n"; }')


## Main Targets

all: directories cache $(STYLESHEETS) dist/.htaccess dist/CHANGES

directories: Makefile generate.pl utils/*
	-rm -rf dist/
	-rm cache
	perl -e 'foreach $$a (qw($(DESTINATION_TYPES))) { print `mkdir -p dist/$$a/tests\n`; foreach $$b (qw($(TEST_TYPES))) { print `mkdir -p dist/$$a/$$b/flat\n`; foreach $$c (qw($(SHELL_TYPES))) { print `mkdir -p dist/$$a/$$b/$$c\n`; } } }'
	mkdir -p dist/style
	touch directories

cache: $(SOURCE)
	perl generate.pl $?


## Support Files

# (not mentioned in 'all' as there is no README)
dist/README: README.dist
	cp README.dist dist/README

dist/CHANGES: CHANGES
	cp CHANGES dist/CHANGES

dist/style/%.css: %.css
	cp $< $@

dist/.htaccess: htaccess
	cp htaccess dist/.htaccess

## Phony Targets

.PHONY: all clean
clean:
	-rm -rf dist/
	-rm directories
	-rm cache

##############################################################################
