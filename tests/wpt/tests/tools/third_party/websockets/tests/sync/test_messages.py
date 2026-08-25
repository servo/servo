import time

from websockets.frames import OP_BINARY, OP_CONT, OP_PING, OP_PONG, OP_TEXT, Frame
from websockets.sync.messages import *

from ..utils import MS
from .utils import ThreadTestCase


class AssemblerTests(ThreadTestCase):
    """
    Tests in this class interact a lot with hidden synchronization mechanisms:

    - get() / get_iter() and put() must run in separate threads when a final
      frame is set because put() waits for get() / get_iter() to fetch the
      message before returning.

    - run_in_thread() lets its target run before yielding back control on entry,
      which guarantees the intended execution order of test cases.

    - run_in_thread() waits for its target to finish running before yielding
      back control on exit, which allows making assertions immediately.

    - When the main thread performs actions that let another thread progress, it
      must wait before making assertions, to avoid depending on scheduling.

    """

    def setUp(self):
        self.assembler = Assembler()

    def tearDown(self):
        """
        Check that the assembler goes back to its default state after each test.

        This removes the need for testing various sequences.

        """
        self.assertFalse(self.assembler.mutex.locked())
        self.assertFalse(self.assembler.get_in_progress)
        self.assertFalse(self.assembler.put_in_progress)
        if not self.assembler.closed:
            self.assertFalse(self.assembler.message_complete.is_set())
            self.assertFalse(self.assembler.message_fetched.is_set())
            self.assertIsNone(self.assembler.decoder)
            self.assertEqual(self.assembler.chunks, [])
            self.assertIsNone(self.assembler.chunks_queue)

    # Test get

    def test_get_text_message_already_received(self):
        """get returns a text message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, "café")

    def test_get_binary_message_already_received(self):
        """get returns a binary message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_BINARY, b"tea"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, b"tea")

    def test_get_text_message_not_received_yet(self):
        """get returns a text message when it is received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        self.assertEqual(message, "café")

    def test_get_binary_message_not_received_yet(self):
        """get returns a binary message when it is received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_BINARY, b"tea"))

        self.assertEqual(message, b"tea")

    def test_get_fragmented_text_message_already_received(self):
        """get reassembles a fragmented a text message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, "café")

    def test_get_fragmented_binary_message_already_received(self):
        """get reassembles a fragmented binary message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            self.assembler.put(Frame(OP_CONT, b"a"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, b"tea")

    def test_get_fragmented_text_message_being_received(self):
        """get reassembles a fragmented text message that is partially received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        self.assertEqual(message, "café")

    def test_get_fragmented_binary_message_being_received(self):
        """get reassembles a fragmented binary message that is partially received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            self.assembler.put(Frame(OP_CONT, b"a"))

        self.assertEqual(message, b"tea")

    def test_get_fragmented_text_message_not_received_yet(self):
        """get reassembles a fragmented text message when it is received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        self.assertEqual(message, "café")

    def test_get_fragmented_binary_message_not_received_yet(self):
        """get reassembles a fragmented binary message when it is received."""
        message = None

        def getter():
            nonlocal message
            message = self.assembler.get()

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            self.assembler.put(Frame(OP_CONT, b"a"))

        self.assertEqual(message, b"tea")

    # Test get_iter

    def test_get_iter_text_message_already_received(self):
        """get_iter yields a text message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        with self.run_in_thread(putter):
            fragments = list(self.assembler.get_iter())

        self.assertEqual(fragments, ["café"])

    def test_get_iter_binary_message_already_received(self):
        """get_iter yields a binary message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_BINARY, b"tea"))

        with self.run_in_thread(putter):
            fragments = list(self.assembler.get_iter())

        self.assertEqual(fragments, [b"tea"])

    def test_get_iter_text_message_not_received_yet(self):
        """get_iter yields a text message when it is received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        self.assertEqual(fragments, ["café"])

    def test_get_iter_binary_message_not_received_yet(self):
        """get_iter yields a binary message when it is received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_BINARY, b"tea"))

        self.assertEqual(fragments, [b"tea"])

    def test_get_iter_fragmented_text_message_already_received(self):
        """get_iter yields a fragmented text message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        with self.run_in_thread(putter):
            fragments = list(self.assembler.get_iter())

        self.assertEqual(fragments, ["ca", "f", "é"])

    def test_get_iter_fragmented_binary_message_already_received(self):
        """get_iter yields a fragmented binary message that is already received."""

        def putter():
            self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            self.assembler.put(Frame(OP_CONT, b"a"))

        with self.run_in_thread(putter):
            fragments = list(self.assembler.get_iter())

        self.assertEqual(fragments, [b"t", b"e", b"a"])

    def test_get_iter_fragmented_text_message_being_received(self):
        """get_iter yields a fragmented text message that is partially received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
        with self.run_in_thread(getter):
            self.assertEqual(fragments, ["ca"])
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, ["ca", "f"])
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        self.assertEqual(fragments, ["ca", "f", "é"])

    def test_get_iter_fragmented_binary_message_being_received(self):
        """get_iter yields a fragmented binary message that is partially received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
        with self.run_in_thread(getter):
            self.assertEqual(fragments, [b"t"])
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, [b"t", b"e"])
            self.assembler.put(Frame(OP_CONT, b"a"))

        self.assertEqual(fragments, [b"t", b"e", b"a"])

    def test_get_iter_fragmented_text_message_not_received_yet(self):
        """get_iter yields a fragmented text message when it is received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_TEXT, b"ca", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, ["ca"])
            self.assembler.put(Frame(OP_CONT, b"f\xc3", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, ["ca", "f"])
            self.assembler.put(Frame(OP_CONT, b"\xa9"))

        self.assertEqual(fragments, ["ca", "f", "é"])

    def test_get_iter_fragmented_binary_message_not_received_yet(self):
        """get_iter yields a fragmented binary message when it is received."""
        fragments = []

        def getter():
            for fragment in self.assembler.get_iter():
                fragments.append(fragment)

        with self.run_in_thread(getter):
            self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, [b"t"])
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            time.sleep(MS)
            self.assertEqual(fragments, [b"t", b"e"])
            self.assembler.put(Frame(OP_CONT, b"a"))

        self.assertEqual(fragments, [b"t", b"e", b"a"])

    # Test timeouts

    def test_get_with_timeout_completes(self):
        """get returns a message when it is received before the timeout."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        with self.run_in_thread(putter):
            message = self.assembler.get(MS)

        self.assertEqual(message, "café")

    def test_get_with_timeout_times_out(self):
        """get raises TimeoutError when no message is received before the timeout."""
        with self.assertRaises(TimeoutError):
            self.assembler.get(MS)

    # Test control frames

    def test_control_frame_before_message_is_ignored(self):
        """get ignores control frames between messages."""

        def putter():
            self.assembler.put(Frame(OP_PING, b""))
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, "café")

    def test_control_frame_in_fragmented_message_is_ignored(self):
        """get ignores control frames within fragmented messages."""

        def putter():
            self.assembler.put(Frame(OP_BINARY, b"t", fin=False))
            self.assembler.put(Frame(OP_PING, b""))
            self.assembler.put(Frame(OP_CONT, b"e", fin=False))
            self.assembler.put(Frame(OP_PONG, b""))
            self.assembler.put(Frame(OP_CONT, b"a"))

        with self.run_in_thread(putter):
            message = self.assembler.get()

        self.assertEqual(message, b"tea")

    # Test concurrency

    def test_get_fails_when_get_is_running(self):
        """get cannot be called concurrently with itself."""
        with self.run_in_thread(self.assembler.get):
            with self.assertRaises(RuntimeError):
                self.assembler.get()
            self.assembler.put(Frame(OP_TEXT, b""))  # unlock other thread

    def test_get_fails_when_get_iter_is_running(self):
        """get cannot be called concurrently with get_iter."""
        with self.run_in_thread(lambda: list(self.assembler.get_iter())):
            with self.assertRaises(RuntimeError):
                self.assembler.get()
            self.assembler.put(Frame(OP_TEXT, b""))  # unlock other thread

    def test_get_iter_fails_when_get_is_running(self):
        """get_iter cannot be called concurrently with get."""
        with self.run_in_thread(self.assembler.get):
            with self.assertRaises(RuntimeError):
                list(self.assembler.get_iter())
            self.assembler.put(Frame(OP_TEXT, b""))  # unlock other thread

    def test_get_iter_fails_when_get_iter_is_running(self):
        """get_iter cannot be called concurrently with itself."""
        with self.run_in_thread(lambda: list(self.assembler.get_iter())):
            with self.assertRaises(RuntimeError):
                list(self.assembler.get_iter())
            self.assembler.put(Frame(OP_TEXT, b""))  # unlock other thread

    def test_put_fails_when_put_is_running(self):
        """put cannot be called concurrently with itself."""

        def putter():
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

        with self.run_in_thread(putter):
            with self.assertRaises(RuntimeError):
                self.assembler.put(Frame(OP_BINARY, b"tea"))
            self.assembler.get()  # unblock other thread

    # Test termination

    def test_get_fails_when_interrupted_by_close(self):
        """get raises EOFError when close is called."""

        def closer():
            time.sleep(2 * MS)
            self.assembler.close()

        with self.run_in_thread(closer):
            with self.assertRaises(EOFError):
                self.assembler.get()

    def test_get_iter_fails_when_interrupted_by_close(self):
        """get_iter raises EOFError when close is called."""

        def closer():
            time.sleep(2 * MS)
            self.assembler.close()

        with self.run_in_thread(closer):
            with self.assertRaises(EOFError):
                list(self.assembler.get_iter())

    def test_put_fails_when_interrupted_by_close(self):
        """put raises EOFError when close is called."""

        def closer():
            time.sleep(2 * MS)
            self.assembler.close()

        with self.run_in_thread(closer):
            with self.assertRaises(EOFError):
                self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

    def test_get_fails_after_close(self):
        """get raises EOFError after close is called."""
        self.assembler.close()
        with self.assertRaises(EOFError):
            self.assembler.get()

    def test_get_iter_fails_after_close(self):
        """get_iter raises EOFError after close is called."""
        self.assembler.close()
        with self.assertRaises(EOFError):
            list(self.assembler.get_iter())

    def test_put_fails_after_close(self):
        """put raises EOFError after close is called."""
        self.assembler.close()
        with self.assertRaises(EOFError):
            self.assembler.put(Frame(OP_TEXT, b"caf\xc3\xa9"))

    def test_close_is_idempotent(self):
        """close can be called multiple times safely."""
        self.assembler.close()
        self.assembler.close()
