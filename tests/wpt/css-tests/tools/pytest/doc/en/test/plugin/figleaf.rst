
report test coverage using the 'figleaf' package.
=================================================


.. contents::
  :local:

Install
---------------

To install the plugin issue::

    easy_install pytest-figleaf  # or
    pip install pytest-figleaf   

and if you are using pip you can also uninstall::

    pip uninstall pytest-figleaf


Usage
---------------

After installation you can simply type::

    py.test --figleaf [...]

to enable figleaf coverage in your test run.  A default ".figleaf" data file
and "html" directory will be created.  You can use command line options
to control where data and html files are created.

command line options
--------------------


``--figleaf``
    trace python coverage with figleaf and write HTML for files below the current working dir
``--fig-data=dir``
    set tracing file, default: ".figleaf".
``--fig-html=dir``
    set html reporting dir, default "html".

.. include:: links.txt
