import time
import threading

from . import mpcontext

"""Instrumentation for measuring high-level time spent on various tasks inside the runner.

This is lower fidelity than an actual profile, but allows custom data to be considered,
so that we can see the time spent in specific tests and test directories.


Instruments are intended to be used as context managers with the return value of __enter__
containing the user-facing API e.g.

with Instrument(*args) as recording:
    recording.set(["init"])
    do_init()
    recording.pause()
    for thread in test_threads:
       thread.start(recording, *args)
    for thread in test_threads:
       thread.join()
    recording.set(["teardown"])   # un-pauses the Instrument
    do_teardown()
"""

class NullInstrument:
    def set(self, stack):
        """Set the current task to stack

        :param stack: A list of strings defining the current task.
                      These are interpreted like a stack trace so that ["foo"] and
                      ["foo", "bar"] both show up as descendants of "foo"
        """
        pass

    def pause(self):
        """Stop recording a task on the current thread. This is useful if the thread
        is purely waiting on the results of other threads"""
        pass

    def __enter__(self):
        return self

    def __exit__(self, *args, **kwargs):
        return


class InstrumentWriter:
    def __init__(self, queue):
        self.queue = queue

    def set(self, stack):
        stack.insert(0, threading.current_thread().name)
        stack = self._check_stack(stack)
        self.queue.put(("set", threading.current_thread().ident, time.time(), stack))

    def pause(self):
        self.queue.put(("pause", threading.current_thread().ident, time.time(), None))

    def _check_stack(self, stack):
        assert isinstance(stack, (tuple, list))
        return [item.replace(" ", "_") for item in stack]


class Instrument:
    def __init__(self, file_path):
        """Instrument that collects data from multiple threads and sums the time in each
        thread. The output is in the format required by flamegraph.pl to enable visualisation
        of the time spent in each task.

        :param file_path: - The path on which to write instrument output. Any existing file
                            at the path will be overwritten
        """
        self.path = file_path
        self.queue = None
        self.current = None
        self.start_time = None
        self.instrument_proc = None

    def __enter__(self):
        assert self.instrument_proc is None
        assert self.queue is None
        mp = mpcontext.get_context()
        self.queue = mp.Queue()
        self.instrument_proc = mp.Process(target=self.run)
        self.instrument_proc.start()
        return InstrumentWriter(self.queue)

    def __exit__(self, *args, **kwargs):
        self.queue.put(("stop", None, time.time(), None))
        self.instrument_proc.join()
        self.instrument_proc = None
        self.queue = None

    def run(self):
        known_commands = {"stop", "pause", "set"}
        with open(self.path, "w") as f:
            thread_data = {}
            while True:
                command, thread, time_stamp, stack = self.queue.get()
                assert command in known_commands

                # If we are done recording, dump the information from all threads to the file
                # before exiting. Otherwise for either 'set' or 'pause' we only need to dump
                # information from the current stack (if any) that was recording on the reporting
                # thread (as that stack is no longer active).
                items = []
                if command == "stop":
                    items = thread_data.values()
                elif thread in thread_data:
                    items.append(thread_data.pop(thread))
                for output_stack, start_time in items:
                    f.write("%s %d\n" % (";".join(output_stack), int(1000 * (time_stamp - start_time))))

                if command == "set":
                    thread_data[thread] = (stack, time_stamp)
                elif command == "stop":
                    break
