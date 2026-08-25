.. include:: ../README.rst

.. module:: atomicwrites

API
===

.. autofunction:: atomic_write


Errorhandling
-------------

All filesystem errors are subclasses of :py:exc:`OSError`.

- On UNIX systems, errors from the Python stdlib calls are thrown.
- On Windows systems, errors from Python's ``ctypes`` are thrown.

In either case, the ``errno`` attribute on the thrown exception maps to an
errorcode in the ``errno`` module.

Low-level API
-------------

.. autofunction:: replace_atomic

.. autofunction:: move_atomic

.. autoclass:: AtomicWriter
   :members:

License
=======

.. include:: ../LICENSE
