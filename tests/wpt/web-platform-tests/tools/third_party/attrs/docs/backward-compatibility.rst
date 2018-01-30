Backward Compatibility
======================

.. currentmodule:: attr

``attrs`` has a very strong backward compatibility policy that is inspired by the policy of the `Twisted framework <https://twistedmatrix.com/trac/wiki/CompatibilityPolicy>`_.

Put simply, you shouldn't ever be afraid to upgrade ``attrs`` if you're only using its public APIs.
If there will ever be a need to break compatibility, it will be announced in the :doc:`changelog` and raise a ``DeprecationWarning`` for a year (if possible) before it's finally really broken.


.. _exemption:

.. warning::

   The structure of the :class:`attr.Attribute` class is exempt from this rule.
   It *will* change in the future, but since it should be considered read-only, that shouldn't matter.

   However if you intend to build extensions on top of ``attrs`` you have to anticipate that.
