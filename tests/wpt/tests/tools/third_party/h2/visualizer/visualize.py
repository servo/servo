# -*- coding: utf-8 -*-
"""
State Machine Visualizer
~~~~~~~~~~~~~~~~~~~~~~~~

This code provides a module that can use graphviz to visualise the state
machines included in hyper-h2. These visualisations can be used as part of the
documentation of hyper-h2, and as a reference material to understand how the
state machines function.

The code in this module is heavily inspired by code in Automat, which can be
found here: https://github.com/glyph/automat. For details on the licensing of
Automat, please see the NOTICES.visualizer file in this folder.

This module is very deliberately not shipped with the rest of hyper-h2. This is
because it is of minimal value to users who are installing hyper-h2: its use
is only really for the developers of hyper-h2.
"""
from __future__ import print_function
import argparse
import collections
import sys

import graphviz
import graphviz.files

import h2.connection
import h2.stream


StateMachine = collections.namedtuple(
    'StateMachine', ['fqdn', 'machine', 'states', 'inputs', 'transitions']
)


# This is all the state machines we currently know about and will render.
# If any new state machines are added, they should be inserted here.
STATE_MACHINES = [
    StateMachine(
        fqdn='h2.connection.H2ConnectionStateMachine',
        machine=h2.connection.H2ConnectionStateMachine,
        states=h2.connection.ConnectionState,
        inputs=h2.connection.ConnectionInputs,
        transitions=h2.connection.H2ConnectionStateMachine._transitions,
    ),
    StateMachine(
        fqdn='h2.stream.H2StreamStateMachine',
        machine=h2.stream.H2StreamStateMachine,
        states=h2.stream.StreamState,
        inputs=h2.stream.StreamInputs,
        transitions=h2.stream._transitions,
    ),
]


def quote(s):
    return '"{}"'.format(s.replace('"', r'\"'))


def html(s):
    return '<{}>'.format(s)


def element(name, *children, **attrs):
    """
    Construct a string from the HTML element description.
    """
    formatted_attributes = ' '.join(
        '{}={}'.format(key, quote(str(value)))
        for key, value in sorted(attrs.items())
    )
    formatted_children = ''.join(children)
    return u'<{name} {attrs}>{children}</{name}>'.format(
        name=name,
        attrs=formatted_attributes,
        children=formatted_children
    )


def row_for_output(event, side_effect):
    """
    Given an output tuple (an event and its side effect), generates a table row
    from it.
    """
    point_size = {'point-size': '9'}
    event_cell = element(
        "td",
        element("font", enum_member_name(event), **point_size)
    )
    side_effect_name = (
        function_name(side_effect) if side_effect is not None else "None"
    )
    side_effect_cell = element(
        "td",
        element("font", side_effect_name, **point_size)
    )
    return element("tr", event_cell, side_effect_cell)


def table_maker(initial_state, final_state, outputs, port):
    """
    Construct an HTML table to label a state transition.
    """
    header = "{} -&gt; {}".format(
        enum_member_name(initial_state), enum_member_name(final_state)
    )
    header_row = element(
        "tr",
        element(
            "td",
            element(
                "font",
                header,
                face="menlo-italic"
            ),
            port=port,
            colspan="2",
        )
    )
    rows = [header_row]
    rows.extend(row_for_output(*output) for output in outputs)
    return element("table", *rows)


def enum_member_name(state):
    """
    All enum member names have the form <EnumClassName>.<EnumMemberName>. For
    our rendering we only want the member name, so we take their representation
    and split it.
    """
    return str(state).split('.', 1)[1]


def function_name(func):
    """
    Given a side-effect function, return its string name.
    """
    return func.__name__


def build_digraph(state_machine):
    """
    Produce a L{graphviz.Digraph} object from a state machine.
    """
    digraph = graphviz.Digraph(node_attr={'fontname': 'Menlo'},
                               edge_attr={'fontname': 'Menlo'},
                               graph_attr={'dpi': '200'})

    # First, add the states as nodes.
    seen_first_state = False
    for state in state_machine.states:
        if not seen_first_state:
            state_shape = "bold"
            font_name = "Menlo-Bold"
        else:
            state_shape = ""
            font_name = "Menlo"
        digraph.node(enum_member_name(state),
                     fontame=font_name,
                     shape="ellipse",
                     style=state_shape,
                     color="blue")
        seen_first_state = True

    # We frequently have vary many inputs that all trigger the same state
    # transition, and only differ in terms of their input and side-effect. It
    # would be polite to say that graphviz does not handle this very well. So
    # instead we *collapse* the state transitions all into the one edge, and
    # then provide a label that displays a table of all the inputs and their
    # associated side effects.
    transitions = collections.defaultdict(list)
    for transition in state_machine.transitions.items():
        initial_state, event = transition[0]
        side_effect, final_state = transition[1]
        transition_key = (initial_state, final_state)
        transitions[transition_key].append((event, side_effect))

    for n, (transition_key, outputs) in enumerate(transitions.items()):
        this_transition = "t{}".format(n)
        initial_state, final_state = transition_key

        port = "tableport"
        table = table_maker(
            initial_state=initial_state,
            final_state=final_state,
            outputs=outputs,
            port=port
        )

        digraph.node(this_transition,
                     label=html(table), margin="0.2", shape="none")

        digraph.edge(enum_member_name(initial_state),
                     '{}:{}:w'.format(this_transition, port),
                     arrowhead="none")
        digraph.edge('{}:{}:e'.format(this_transition, port),
                     enum_member_name(final_state))

    return digraph


def main():
    """
    Renders all the state machines in hyper-h2 into images.
    """
    program_name = sys.argv[0]
    argv = sys.argv[1:]

    description = """
    Visualize hyper-h2 state machines as graphs.
    """
    epilog = """
    You must have the graphviz tool suite installed.  Please visit
    http://www.graphviz.org for more information.
    """

    argument_parser = argparse.ArgumentParser(
        prog=program_name,
        description=description,
        epilog=epilog
    )
    argument_parser.add_argument(
        '--image-directory',
        '-i',
        help="Where to write out image files.",
        default=".h2_visualize"
    )
    argument_parser.add_argument(
        '--view',
        '-v',
        help="View rendered graphs with default image viewer",
        default=False,
        action="store_true"
    )
    args = argument_parser.parse_args(argv)

    for state_machine in STATE_MACHINES:
        print(state_machine.fqdn, '...discovered')

        digraph = build_digraph(state_machine)

        if args.image_directory:
            digraph.format = "png"
            digraph.render(filename="{}.dot".format(state_machine.fqdn),
                           directory=args.image_directory,
                           view=args.view,
                           cleanup=True)
            print(state_machine.fqdn, "...wrote image into", args.image_directory)


if __name__ == '__main__':
    main()
