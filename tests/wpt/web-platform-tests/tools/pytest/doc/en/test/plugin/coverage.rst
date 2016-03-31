
Write and report coverage data with the 'coverage' package.
===========================================================


.. contents::
  :local:

Note: Original code by Ross Lawley. 

Install
--------------

Use pip to (un)install::

    pip install pytest-coverage 
    pip uninstall pytest-coverage 

or alternatively use easy_install to install::

    easy_install pytest-coverage 


Usage 
-------------

To get full test coverage reports for a particular package type::

    py.test --cover-report=report

command line options
--------------------


``--cover=COVERPACKAGES``
    (multi allowed) only include info from specified package.
``--cover-report=REPORT_TYPE``
    html: Directory for html output.
                    report: Output a text report.
                    annotate: Annotate your source code for which lines were executed and which were not.
                    xml: Output an xml report compatible with the cobertura plugin for hudson.
``--cover-directory=DIRECTORY``
    Directory for the reports (html / annotate results) defaults to ./coverage
``--cover-xml-file=XML_FILE``
    File for the xml report defaults to ./coverage.xml
``--cover-show-missing``
    Show missing files
``--cover-ignore-errors=IGNORE_ERRORS``
    Ignore errors of finding source files for code.

.. include:: links.txt
