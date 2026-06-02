Annotation-Protocol: Guidelines for Contributing Tests
======================================================

This document describes the method people should use for authoring tests and
integrating them into the repository.  Anyone is welcome to submit new tests to
this collection.  If you do, please create the tests following the guidelines
below.  Then submit them as a pull request so they can be evaluated

Structure
---------

Tests are organized by client or server, and then by major section of the Annotation
Protocol specification.  The folders associated with these are:

* client - tests a client needs to run
* server - tests to be run against a server

Within these folders, special files ending with the suffix ".html" provide the source
for the test as a set javascript calls to perform the test.

* scripts - JavaScript that are included by tests
* tools - supporting scripts and files

Client Test Cases
-----------------

@@@TODO@@@ describe the structure of client test cases.

Server Test Cases
-----------------

@@@TODO@@@ describe the structure of server test cases.


Command Line Tools
------------------

### Stand-alone Annotation Server ###
