_LOCK_KEY = "network-error-logging:lock"
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
    if lock_holder != request_id:
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
