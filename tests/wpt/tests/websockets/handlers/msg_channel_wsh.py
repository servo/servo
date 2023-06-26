#!/usr/bin/python
import json
import logging
import urllib
import threading
import traceback
from queue import Empty

from mod_pywebsocket import stream, msgutil
from wptserve import stash as stashmod

logger = logging.getLogger()

address, authkey = stashmod.load_env_config()
stash = stashmod.Stash("msg_channel", address=address, authkey=authkey)

# Backend for websocket based channels.
#
# Each socket connection has a uuid identifying the channel and a
# direction which is either "read" or "write".  There can be only 1
# "read" channel per uuid, but multiple "write" channels
# (i.e. multiple producer, single consumer).
#
# The websocket connection URL contains the uuid and the direction as
# named query parameters.
#
# Channels are backed by a queue which is stored in the stash (one
# queue per uuid).
#
# The representation of a queue in the stash is a tuple (queue,
# has_reader, writer_count).  The first field is the queue itself, the
# latter are effectively reference counts for reader channels (which
# is zero or one, represented by a bool) and writer channels.  Once
# both counts drop to zero the queue can be deleted.
#
# Entries on the queue itself are formed of (command, data) pairs. The
# command can be either "close", signalling the socket is closing and
# the reference count on the channel should be decremented, or
# "message", which indicates a message.


def log(uuid, msg, level="debug"):
    msg = f"{uuid}: {msg}"
    getattr(logger, level)(msg)


def web_socket_do_extra_handshake(request):
    return


def web_socket_transfer_data(request):
    """Handle opening a websocket connection."""

    uuid, direction = parse_request(request)
    log(uuid, f"Got web_socket_transfer_data {direction}")

    # Get or create the relevant queue from the stash and update the refcount
    with stash.lock:
        value = stash.take(uuid)
        if value is None:
            queue = stash.get_queue()
            if direction == "read":
                has_reader = True
                writer_count = 0
            else:
                has_reader = False
                writer_count = 1
        else:
            queue, has_reader, writer_count = value
            if direction == "read":
                if has_reader:
                    raise ValueError("Tried to start multiple readers for the same queue")
                has_reader = True
            else:
                writer_count += 1

        stash.put(uuid, (queue, has_reader, writer_count))

    if direction == "read":
        run_read(request, uuid, queue)
    elif direction == "write":
        run_write(request, uuid, queue)

    log(uuid, f"transfer_data loop exited {direction}")
    close_channel(uuid, direction)


def web_socket_passive_closing_handshake(request):
    """Handle a client initiated close.

    When the client closes a reader, put a message in the message
    queue indicating the close. For a writer we don't need special
    handling here because receive_message in run_read will return an
    empty message in this case, so that loop will exit on its own.
    """
    uuid, direction = parse_request(request)
    log(uuid, f"Got web_socket_passive_closing_handshake {direction}")

    if direction == "read":
        with stash.lock:
            data = stash.take(uuid)
            stash.put(uuid, data)
        if data is not None:
            queue = data[0]
            queue.put(("close", None))

    return request.ws_close_code, request.ws_close_reason


def parse_request(request):
    query = request.unparsed_uri.split('?')[1]
    GET = dict(urllib.parse.parse_qsl(query))
    uuid = GET["uuid"]
    direction = GET["direction"]
    return uuid, direction


def wait_for_close(request, uuid, queue):
    """Listen for messages on the socket for a read connection to a channel."""
    closed = False
    while not closed:
        try:
            msg = request.ws_stream.receive_message()
            if msg is None:
                break
            try:
                cmd, data = json.loads(msg)
            except ValueError:
                cmd = None
            if cmd == "close":
                closed = True
                log(uuid, "Got client initiated close")
            else:
                log(uuid, f"Unexpected message on read socket {msg}", "warning")
        except Exception:
            if not (request.server_terminated or request.client_terminated):
                log(uuid, f"Got exception in wait_for_close\n{traceback.format_exc()}")
            closed = True

    if not request.server_terminated:
        queue.put(("close", None))


def run_read(request, uuid, queue):
    """Main loop for a read-type connection.

    This mostly just listens on the queue for new messages of the
    form (message, data). Supported messages are:
     message - Send `data` on the WebSocket
     close - Close the reader queue

    In addition there's a thread that listens for messages on the
    socket itself. Typically this socket shouldn't recieve any
    messages, but it can recieve an explicit "close" message,
    indicating the socket should be disconnected.
    """

    close_thread = threading.Thread(target=wait_for_close, args=(request, uuid, queue), daemon=True)
    close_thread.start()

    while True:
        try:
            data = queue.get(True, 1)
        except Empty:
            if request.server_terminated or request.client_terminated:
                break
        else:
            cmd, body = data
            log(uuid, f"queue.get ({cmd}, {body})")
            if cmd == "close":
                break
            if cmd == "message":
                msgutil.send_message(request, json.dumps(body))
            else:
                log(uuid, f"Unknown queue command {cmd}", level="warning")


def run_write(request, uuid, queue):
    """Main loop for a write-type connection.

    Messages coming over the socket have the format (command, data).
    The recognised commands are:
     message - Send the message `data` over the channel.
     disconnectReader - Close the reader connection for this channel.
     delete - Force-delete the entire channel and the underlying queue.
    """
    while True:
        msg = request.ws_stream.receive_message()
        if msg is None:
            break
        cmd, body = json.loads(msg)
        if cmd == "disconnectReader":
            queue.put(("close", None))
        elif cmd == "message":
            log(uuid, f"queue.put ({cmd}, {body})")
            queue.put((cmd, body))
        elif cmd == "delete":
            close_channel(uuid, None)


def close_channel(uuid, direction):
    """Update the channel state in the stash when closing a connection

    This updates the stash entry, including refcounts, once a
    connection to a channel is closed.

    Params:
    uuid - the UUID of the channel being closed.
    direction - "read" if a read connection was closed, "write" if a
                write connection was closed, None to remove the
                underlying queue from the stash entirely.

    """
    log(uuid, f"Got close_channel {direction}")
    with stash.lock:
        data = stash.take(uuid)
        if data is None:
            log(uuid, "Message queue already deleted")
            return
        if direction is None:
            # Return without replacing the channel in the stash
            log(uuid, "Force deleting message queue")
            return
        queue, has_reader, writer_count = data
        if direction == "read":
            has_reader = False
        else:
            writer_count -= 1

        if has_reader or writer_count > 0 or not queue.empty():
            log(uuid, f"Updating refcount {has_reader}, {writer_count}")
            stash.put(uuid, (queue, has_reader, writer_count))
        else:
            log(uuid, "Deleting message queue")
