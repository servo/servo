hyperframe: HTTP/2 Framing for Python
=====================================

hyperframe is a pure-Python tool for working with HTTP/2 frames. This library
allows you to create, serialize, and parse HTTP/2 frames.

Working with it is easy:

.. code-block:: python

    import hyperframe.frame

    f = hyperframe.frame.DataFrame(stream_id=5)
    f.data = b'some binary data'
    f.flags.add('END_STREAM')
    f.flags.add('PADDED')
    f.padding_length = 30

    data = f.serialize()

    new_frame, length = hyperframe.frame.Frame.parse_frame_header(data[:9])
    new_frame.parse_body(memoryview(data[9:9 + length]))

hyperframe is pure-Python, contains no external dependencies, and runs on a
wide variety of Python interpreters and platforms. Made available under the MIT
license, why write your own frame parser?

Contents:

.. toctree::
   :maxdepth: 2

   installation
   api
