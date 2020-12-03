from tests.support.sync import Poll


def wait_for_new_handle(session, handles_before):
    def find_new_handle(session):
        new_handles = list(set(session.handles) - set(handles_before))
        if new_handles and len(new_handles) == 1:
            return new_handles[0]
        return None

    wait = Poll(
        session,
        timeout=5,
        message="No new window has been opened")

    return wait.until(find_new_handle)
