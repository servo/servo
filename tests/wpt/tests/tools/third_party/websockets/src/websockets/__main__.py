from __future__ import annotations

import argparse
import asyncio
import os
import signal
import sys
import threading
from typing import Any, Set

from .exceptions import ConnectionClosed
from .frames import Close
from .legacy.client import connect
from .version import version as websockets_version


if sys.platform == "win32":

    def win_enable_vt100() -> None:
        """
        Enable VT-100 for console output on Windows.

        See also https://bugs.python.org/issue29059.

        """
        import ctypes

        STD_OUTPUT_HANDLE = ctypes.c_uint(-11)
        INVALID_HANDLE_VALUE = ctypes.c_uint(-1)
        ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x004

        handle = ctypes.windll.kernel32.GetStdHandle(STD_OUTPUT_HANDLE)
        if handle == INVALID_HANDLE_VALUE:
            raise RuntimeError("unable to obtain stdout handle")

        cur_mode = ctypes.c_uint()
        if ctypes.windll.kernel32.GetConsoleMode(handle, ctypes.byref(cur_mode)) == 0:
            raise RuntimeError("unable to query current console mode")

        # ctypes ints lack support for the required bit-OR operation.
        # Temporarily convert to Py int, do the OR and convert back.
        py_int_mode = int.from_bytes(cur_mode, sys.byteorder)
        new_mode = ctypes.c_uint(py_int_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)

        if ctypes.windll.kernel32.SetConsoleMode(handle, new_mode) == 0:
            raise RuntimeError("unable to set console mode")


def exit_from_event_loop_thread(
    loop: asyncio.AbstractEventLoop,
    stop: asyncio.Future[None],
) -> None:
    loop.stop()
    if not stop.done():
        # When exiting the thread that runs the event loop, raise
        # KeyboardInterrupt in the main thread to exit the program.
        if sys.platform == "win32":
            ctrl_c = signal.CTRL_C_EVENT
        else:
            ctrl_c = signal.SIGINT
        os.kill(os.getpid(), ctrl_c)


def print_during_input(string: str) -> None:
    sys.stdout.write(
        # Save cursor position
        "\N{ESC}7"
        # Add a new line
        "\N{LINE FEED}"
        # Move cursor up
        "\N{ESC}[A"
        # Insert blank line, scroll last line down
        "\N{ESC}[L"
        # Print string in the inserted blank line
        f"{string}\N{LINE FEED}"
        # Restore cursor position
        "\N{ESC}8"
        # Move cursor down
        "\N{ESC}[B"
    )
    sys.stdout.flush()


def print_over_input(string: str) -> None:
    sys.stdout.write(
        # Move cursor to beginning of line
        "\N{CARRIAGE RETURN}"
        # Delete current line
        "\N{ESC}[K"
        # Print string
        f"{string}\N{LINE FEED}"
    )
    sys.stdout.flush()


async def run_client(
    uri: str,
    loop: asyncio.AbstractEventLoop,
    inputs: asyncio.Queue[str],
    stop: asyncio.Future[None],
) -> None:
    try:
        websocket = await connect(uri)
    except Exception as exc:
        print_over_input(f"Failed to connect to {uri}: {exc}.")
        exit_from_event_loop_thread(loop, stop)
        return
    else:
        print_during_input(f"Connected to {uri}.")

    try:
        while True:
            incoming: asyncio.Future[Any] = asyncio.create_task(websocket.recv())
            outgoing: asyncio.Future[Any] = asyncio.create_task(inputs.get())
            done: Set[asyncio.Future[Any]]
            pending: Set[asyncio.Future[Any]]
            done, pending = await asyncio.wait(
                [incoming, outgoing, stop], return_when=asyncio.FIRST_COMPLETED
            )

            # Cancel pending tasks to avoid leaking them.
            if incoming in pending:
                incoming.cancel()
            if outgoing in pending:
                outgoing.cancel()

            if incoming in done:
                try:
                    message = incoming.result()
                except ConnectionClosed:
                    break
                else:
                    if isinstance(message, str):
                        print_during_input("< " + message)
                    else:
                        print_during_input("< (binary) " + message.hex())

            if outgoing in done:
                message = outgoing.result()
                await websocket.send(message)

            if stop in done:
                break

    finally:
        await websocket.close()
        assert websocket.close_code is not None and websocket.close_reason is not None
        close_status = Close(websocket.close_code, websocket.close_reason)

        print_over_input(f"Connection closed: {close_status}.")

        exit_from_event_loop_thread(loop, stop)


def main() -> None:
    # Parse command line arguments.
    parser = argparse.ArgumentParser(
        prog="python -m websockets",
        description="Interactive WebSocket client.",
        add_help=False,
    )
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--version", action="store_true")
    group.add_argument("uri", metavar="<uri>", nargs="?")
    args = parser.parse_args()

    if args.version:
        print(f"websockets {websockets_version}")
        return

    if args.uri is None:
        parser.error("the following arguments are required: <uri>")

    # If we're on Windows, enable VT100 terminal support.
    if sys.platform == "win32":
        try:
            win_enable_vt100()
        except RuntimeError as exc:
            sys.stderr.write(
                f"Unable to set terminal to VT100 mode. This is only "
                f"supported since Win10 anniversary update. Expect "
                f"weird symbols on the terminal.\nError: {exc}\n"
            )
            sys.stderr.flush()

    try:
        import readline  # noqa
    except ImportError:  # Windows has no `readline` normally
        pass

    # Create an event loop that will run in a background thread.
    loop = asyncio.new_event_loop()

    # Due to zealous removal of the loop parameter in the Queue constructor,
    # we need a factory coroutine to run in the freshly created event loop.
    async def queue_factory() -> asyncio.Queue[str]:
        return asyncio.Queue()

    # Create a queue of user inputs. There's no need to limit its size.
    inputs: asyncio.Queue[str] = loop.run_until_complete(queue_factory())

    # Create a stop condition when receiving SIGINT or SIGTERM.
    stop: asyncio.Future[None] = loop.create_future()

    # Schedule the task that will manage the connection.
    loop.create_task(run_client(args.uri, loop, inputs, stop))

    # Start the event loop in a background thread.
    thread = threading.Thread(target=loop.run_forever)
    thread.start()

    # Read from stdin in the main thread in order to receive signals.
    try:
        while True:
            # Since there's no size limit, put_nowait is identical to put.
            message = input("> ")
            loop.call_soon_threadsafe(inputs.put_nowait, message)
    except (KeyboardInterrupt, EOFError):  # ^C, ^D
        loop.call_soon_threadsafe(stop.set_result, None)

    # Wait for the event loop to terminate.
    thread.join()

    # For reasons unclear, even though the loop is closed in the thread,
    # it still thinks it's running here.
    loop.close()


if __name__ == "__main__":
    main()
