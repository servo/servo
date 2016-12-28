Vocabulary Exercising Tools
===========================

The Web Annotation Vocabulary specification defines an ontology for
portable annotations.  The tools in this directory, along with the
sample files supplied, can be used to exercise the vocabulary
"implementation" against various RDF processing engines.

ruby-rdf
========

This directory contains a Ruby script that will evaluate the samples.  See
the README.md file in that directory for more information.

vocab-tester.py
===============

This python script exercises the vocabulary implementation using rdflib,
rdflib-jsonld and pyld.  Note that this means your python environment must
have those additional python modules installed in order to run the tests.

