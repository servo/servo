# This file implements a shared lock that lets us ensure that the test cases in
# this directory run serially.  Each test case obtains this lock as its first
# step, and releases it as its last.  (The nel_test helper function in
# nel.sub.js automates this process.)  Because the lock needs to be shared
# across all of the test cases, we use a hard-coded stash key.  This hard-coded
# key is a random UUID, which should not conflict with any other auto-generated
# stash keys.

import time

_LOCK_KEY = "67966d2e-a847-41d8-b7c3-5f6aee3375ba"
_TIMEOUT = 5  # seconds

def wait_for_lock(request):
  t0 = time.time()
  while time.time() - t0 < _TIMEOUT:
    time.sleep(0.5)
    value = request.server.stash.take(key=_LOCK_KEY)
    if value is None:
      return True
  return False

def lock(request, report_id):
  with request.server.stash.lock:
    # Loop until the lock is free
    if not wait_for_lock(request):
      return (503, [], "Cannot obtain lock")
    request.server.stash.put(key=_LOCK_KEY, value=report_id)
    return "Obtained lock for %s" % report_id

def unlock(request, report_id):
  with request.server.stash.lock:
    lock_holder = request.server.stash.take(key=_LOCK_KEY)
    if lock_holder != report_id:
      # Return the lock holder to the stash
      request.server.stash.put(key=_LOCK_KEY, value=lock_holder)
      return (503, [], "Cannot release lock held by %s" % lock_holder)
  return "Released lock for %s" % report_id

def main(request, response):
  op = request.GET.first("op")
  report_id = request.GET.first("reportID")
  if op == "lock":
    return lock(request, report_id)
  elif op == "unlock":
    return unlock(request, report_id)
  else:
    return (400, [], "Invalid op")
