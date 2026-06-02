hyperframe API
==============

This document provides the hyperframe API.

All frame classes are subclasses of :class:`Frame <hyperframe.frame.Frame>`,
and provide the methods and attributes defined there. Additionally, some frames
use the :class:`Priority <hyperframe.frame.Priority>` and
:class:`Padding <hyperframe.frame.Padding>` mixins, and make the methods and
attributes defined on *those* mixins available as well.

Rather than clutter up the documentation repeatedly documenting those methods
and objects, we explicitly show the inheritance hierarchy of the frames: don't
forget to consult the parent classes before deciding if a method or attribute
you need is not present!

.. autoclass:: hyperframe.frame.Frame
   :members:

.. autoclass:: hyperframe.frame.Padding
   :members:

.. autoclass:: hyperframe.frame.Priority
   :members:

.. autoclass:: hyperframe.frame.DataFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.PriorityFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.RstStreamFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.SettingsFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.PushPromiseFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.PingFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.GoAwayFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.WindowUpdateFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.HeadersFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.ContinuationFrame
   :show-inheritance:
   :members:

.. autoclass:: hyperframe.frame.ExtensionFrame
   :show-inheritance:
   :members:

.. autodata:: hyperframe.frame.FRAMES

Exceptions
----------

.. autoclass:: hyperframe.exceptions.HyperframeError
   :members:

.. autoclass:: hyperframe.exceptions.UnknownFrameError
   :members:

.. autoclass:: hyperframe.exceptions.InvalidPaddingError
   :members:

.. autoclass:: hyperframe.exceptions.InvalidFrameError
   :members:

.. autoclass:: hyperframe.exceptions.InvalidDataError
   :members:
