Stash
=====

Object for storing cross-request state. This is unusual in that keys
must be UUIDs, in order to prevent different clients setting the same
key, and values are write-once, read-once to minimise the chances of
state persisting indefinitely. The stash defines two operations;
`put`, to add state and `take` to remove state. Furthermore, the view
of the stash is path-specific; by default a request will only see the
part of the stash corresponding to its own path.

A typical example of using a stash to store state might be::

  @handler
  def handler(request, response):
      # We assume this is a string representing a UUID
      key = request.GET.first("id")

      if request.method == "POST":
          request.server.stash.put(key, "Some sample value")
          return "Added value to stash"
      else:
          value = request.server.stash.take(key)
          assert request.server.stash.take(key) is None
          return key

:mod:`Interface <stash>`
------------------------

.. automodule:: wptserve.stash
   :members:
