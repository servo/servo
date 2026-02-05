/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{RefCell, RefMut};
use std::{cmp, fmt, hash};

use petgraph::Direction;
use petgraph::graph::DefaultIx;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{DfsPostOrder, EdgeRef, Reversed};
use smallvec::SmallVec;

use crate::block::{Block, Chunk};
use crate::destination_node::DestinationNode;
use crate::listener::AudioListenerNode;
use crate::node::{AudioNodeEngine, BlockInfo, ChannelCountMode, ChannelInterpretation};
use crate::param::ParamType;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
/// A unique identifier for nodes in the graph. Stable
/// under graph mutation.
pub struct NodeId(NodeIndex<DefaultIx>);

impl NodeId {
    pub fn input(self, port: u32) -> PortId<InputPort> {
        PortId(self, PortIndex::Port(port))
    }
    pub fn param(self, param: ParamType) -> PortId<InputPort> {
        PortId(self, PortIndex::Param(param))
    }
    pub fn output(self, port: u32) -> PortId<OutputPort> {
        PortId(self, PortIndex::Port(port))
    }
    pub(crate) fn listener(self) -> PortId<InputPort> {
        PortId(self, PortIndex::Listener(()))
    }
}

/// A zero-indexed "port" for a node. Most nodes have one
/// input and one output port, but some may have more.
/// For example, a channel splitter node will have one output
/// port for each channel.
///
/// These are essentially indices into the Chunks
///
/// Kind is a zero sized type and is useful for distinguishing
/// between input and output ports (which may otherwise share indices)
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub enum PortIndex<Kind: PortKind> {
    Port(u32),
    Param(Kind::ParamId),
    /// special variant only used for the implicit connection
    /// from listeners to params
    Listener(Kind::Listener),
}

impl<Kind: PortKind> PortId<Kind> {
    pub fn node(&self) -> NodeId {
        self.0
    }
}

pub trait PortKind {
    type ParamId: Copy + Eq + PartialEq + Ord + PartialOrd + hash::Hash + fmt::Debug;
    type Listener: Copy + Eq + PartialEq + Ord + PartialOrd + hash::Hash + fmt::Debug;
}

/// An identifier for a port.
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub struct PortId<Kind: PortKind>(NodeId, PortIndex<Kind>);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
/// Marker type for denoting that the port is an input port
/// of the node it is connected to
pub struct InputPort;
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
/// Marker type for denoting that the port is an output port
/// of the node it is connected to
pub struct OutputPort;

impl PortKind for InputPort {
    type ParamId = ParamType;
    type Listener = ();
}

#[derive(Debug, Hash, PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
pub enum Void {}

impl PortKind for OutputPort {
    // Params are only a feature of input ports. By using an empty type here
    // we ensure that the PortIndex enum has zero overhead for outputs,
    // taking up no extra discriminant space and eliminating PortIndex::Param
    // branches entirely from the compiled code
    type ParamId = Void;
    type Listener = Void;
}

pub struct AudioGraph {
    graph: StableGraph<Node, Edge>,
    dest_id: NodeId,
    dests: Vec<NodeId>,
    listener_id: NodeId,
}

pub(crate) struct Node {
    node: RefCell<Box<dyn AudioNodeEngine>>,
}

/// An edge in the graph
///
/// This connects one or more pair of ports between two
/// nodes, each connection represented by a `Connection`.
/// WebAudio allows for multiple connections to/from the same port
/// however it does not allow for duplicate connections between pairs
/// of ports
pub(crate) struct Edge {
    connections: SmallVec<[Connection; 1]>,
}

impl Edge {
    /// Find if there are connections between two given ports, return the index
    fn has_between(
        &self,
        output_idx: PortIndex<OutputPort>,
        input_idx: PortIndex<InputPort>,
    ) -> bool {
        self.connections
            .iter()
            .any(|e| e.input_idx == input_idx && e.output_idx == output_idx)
    }

    fn remove_by_output(&mut self, output_idx: PortIndex<OutputPort>) {
        self.connections.retain(|i| i.output_idx != output_idx)
    }

    fn remove_by_input(&mut self, input_idx: PortIndex<InputPort>) {
        self.connections.retain(|i| i.input_idx != input_idx)
    }

    fn remove_by_pair(
        &mut self,
        output_idx: PortIndex<OutputPort>,
        input_idx: PortIndex<InputPort>,
    ) {
        self.connections
            .retain(|i| i.output_idx != output_idx || i.input_idx != input_idx)
    }
}

/// A single connection between ports
struct Connection {
    /// The index of the port on the input node
    /// This is actually the /output/ of this edge
    input_idx: PortIndex<InputPort>,
    /// The index of the port on the output node
    /// This is actually the /input/ of this edge
    output_idx: PortIndex<OutputPort>,
    /// When the from node finishes processing, it will push
    /// its data into this cache for the input node to read
    cache: RefCell<Option<Block>>,
}

impl AudioGraph {
    pub fn new(channel_count: u8) -> Self {
        let mut graph = StableGraph::new();
        let dest_id =
            NodeId(graph.add_node(Node::new(Box::new(DestinationNode::new(channel_count)))));
        let listener_id = NodeId(graph.add_node(Node::new(Box::new(AudioListenerNode::new()))));
        AudioGraph {
            graph,
            dest_id,
            dests: vec![dest_id],
            listener_id,
        }
    }

    /// Create a node, obtain its id
    pub(crate) fn add_node(&mut self, node: Box<dyn AudioNodeEngine>) -> NodeId {
        NodeId(self.graph.add_node(Node::new(node)))
    }

    /// Connect an output port to an input port
    ///
    /// The edge goes *from* the output port *to* the input port, connecting two nodes
    pub fn add_edge(&mut self, out: PortId<OutputPort>, inp: PortId<InputPort>) {
        let edge = self
            .graph
            .edges(out.node().0)
            .find(|e| e.target() == inp.node().0)
            .map(|e| e.id());
        if let Some(e) = edge {
            // .find(|e| e.weight().has_between(out.1, inp.1));
            let w = self
                .graph
                .edge_weight_mut(e)
                .expect("This edge is known to exist");
            if w.has_between(out.1, inp.1) {
                return;
            }
            w.connections.push(Connection::new(inp.1, out.1))
        } else {
            // add a new edge
            self.graph
                .add_edge(out.node().0, inp.node().0, Edge::new(inp.1, out.1));
        }
    }

    /// Disconnect all outgoing connections from a node
    ///
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect
    pub fn disconnect_all_from(&mut self, node: NodeId) {
        let edges = self.graph.edges(node.0).map(|e| e.id()).collect::<Vec<_>>();
        for edge in edges {
            self.graph.remove_edge(edge);
        }
    }

    // /// Disconnect all outgoing connections from a node's output
    // ///
    // /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-output
    pub fn disconnect_output(&mut self, out: PortId<OutputPort>) {
        let candidates: Vec<_> = self
            .graph
            .edges(out.node().0)
            .map(|e| (e.id(), e.target()))
            .collect();
        for (edge, to) in candidates {
            let mut e = self
                .graph
                .remove_edge(edge)
                .expect("Edge index is known to exist");
            e.remove_by_output(out.1);
            if !e.connections.is_empty() {
                self.graph.add_edge(out.node().0, to, e);
            }
        }
    }

    /// Disconnect connections from a node to another node
    ///
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode
    pub fn disconnect_between(&mut self, from: NodeId, to: NodeId) {
        let edge = self
            .graph
            .edges(from.0)
            .find(|e| e.target() == to.0)
            .map(|e| e.id());
        if let Some(i) = edge {
            self.graph.remove_edge(i);
        }
    }

    /// Disconnect all outgoing connections from a node's output to another node
    ///
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output
    pub fn disconnect_output_between(&mut self, out: PortId<OutputPort>, to: NodeId) {
        let edge = self
            .graph
            .edges(out.node().0)
            .find(|e| e.target() == to.0)
            .map(|e| e.id());
        if let Some(edge) = edge {
            let mut e = self
                .graph
                .remove_edge(edge)
                .expect("Edge index is known to exist");
            e.remove_by_output(out.1);
            if !e.connections.is_empty() {
                self.graph.add_edge(out.node().0, to.0, e);
            }
        }
    }

    /// Disconnect all outgoing connections from a node to another node's input
    ///
    /// Only used in WebAudio for disconnecting audio params
    ///
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationparam
    pub fn disconnect_to(&mut self, node: NodeId, inp: PortId<InputPort>) {
        let edge = self
            .graph
            .edges(node.0)
            .find(|e| e.target() == inp.node().0)
            .map(|e| e.id());
        if let Some(edge) = edge {
            let mut e = self
                .graph
                .remove_edge(edge)
                .expect("Edge index is known to exist");
            e.remove_by_input(inp.1);
            if !e.connections.is_empty() {
                self.graph.add_edge(node.0, inp.node().0, e);
            }
        }
    }

    /// Disconnect all outgoing connections from a node's output to another node's input
    ///
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output-input
    /// https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationparam-output
    pub fn disconnect_output_between_to(
        &mut self,
        out: PortId<OutputPort>,
        inp: PortId<InputPort>,
    ) {
        let edge = self
            .graph
            .edges(out.node().0)
            .find(|e| e.target() == inp.node().0)
            .map(|e| e.id());
        if let Some(edge) = edge {
            let mut e = self
                .graph
                .remove_edge(edge)
                .expect("Edge index is known to exist");
            e.remove_by_pair(out.1, inp.1);
            if !e.connections.is_empty() {
                self.graph.add_edge(out.node().0, inp.node().0, e);
            }
        }
    }

    /// Get the id of the destination node in this graph
    ///
    /// All graphs have a destination node, with one input port
    pub fn dest_id(&self) -> NodeId {
        self.dest_id
    }

    /// Add additional terminator nodes
    pub fn add_extra_dest(&mut self, dest: NodeId) {
        self.dests.push(dest);
    }

    /// Get the id of the AudioListener in this graph
    ///
    /// All graphs have a single listener, with no ports (but nine AudioParams)
    ///
    /// N.B. The listener actually has a single output port containing
    /// its position data for the block, however this should
    /// not be exposed to the DOM.
    pub fn listener_id(&self) -> NodeId {
        self.listener_id
    }

    /// For a given block, process all the data on this graph
    pub fn process(&mut self, info: &BlockInfo) -> Chunk {
        // DFS post order: Children are processed before their parent,
        // which is exactly what we need since the parent depends on the
        // children's output
        //
        // This will only visit each node once
        let reversed = Reversed(&self.graph);

        let mut blocks: SmallVec<[SmallVec<[Block; 1]>; 1]> = SmallVec::new();
        let mut output_counts: SmallVec<[u32; 1]> = SmallVec::new();

        let mut visit = DfsPostOrder::empty(reversed);

        for dest in &self.dests {
            visit.move_to(dest.0);

            while let Some(ix) = visit.next(reversed) {
                let mut curr = self.graph[ix].node.borrow_mut();

                let mut chunk = Chunk::default();
                chunk
                    .blocks
                    .resize(curr.input_count() as usize, Default::default());

                // if we have inputs, collect all the computed blocks
                // and construct a Chunk

                // set up scratch space to store all the blocks
                blocks.clear();
                blocks.resize(curr.input_count() as usize, Default::default());

                let mode = curr.channel_count_mode();
                let count = curr.channel_count();
                let interpretation = curr.channel_interpretation();

                // all edges to this node are from its dependencies
                for edge in self.graph.edges_directed(ix, Direction::Incoming) {
                    let edge = edge.weight();
                    for connection in &edge.connections {
                        let mut block = connection
                            .cache
                            .borrow_mut()
                            .take()
                            .expect("Cache should have been filled from traversal");

                        match connection.input_idx {
                            PortIndex::Port(idx) => {
                                blocks[idx as usize].push(block);
                            },
                            PortIndex::Param(param) => {
                                // param inputs are downmixed to mono
                                // https://webaudio.github.io/web-audio-api/#dom-audionode-connect-destinationparam-output
                                block.mix(1, ChannelInterpretation::Speakers);
                                curr.get_param(param).add_block(block)
                            },
                            PortIndex::Listener(_) => curr.set_listenerdata(block),
                        }
                    }
                }

                for (i, mut blocks) in blocks.drain(..).enumerate() {
                    if blocks.is_empty() {
                        if mode == ChannelCountMode::Explicit {
                            // It's silence, but mix it anyway
                            chunk.blocks[i].mix(count, interpretation);
                        }
                    } else if blocks.len() == 1 {
                        chunk.blocks[i] = blocks.pop().expect("`blocks` had length 1");
                        match mode {
                            ChannelCountMode::Explicit => {
                                chunk.blocks[i].mix(count, interpretation);
                            },
                            ChannelCountMode::ClampedMax => {
                                if chunk.blocks[i].chan_count() > count {
                                    chunk.blocks[i].mix(count, interpretation);
                                }
                            },
                            // It's one channel, it maxes itself
                            ChannelCountMode::Max => (),
                        }
                    } else {
                        let mix_count = match mode {
                            ChannelCountMode::Explicit => count,
                            _ => {
                                let mut max = 0; // max channel count
                                for block in &blocks {
                                    max = cmp::max(max, block.chan_count());
                                }
                                if mode == ChannelCountMode::ClampedMax {
                                    max = cmp::min(max, count);
                                }
                                max
                            },
                        };
                        let block = blocks.into_iter().fold(Block::default(), |acc, mut block| {
                            block.mix(mix_count, interpretation);
                            acc.sum(block)
                        });
                        chunk.blocks[i] = block;
                    }
                }

                // actually run the node engine
                let mut out = curr.process(chunk, info);

                assert_eq!(out.len(), curr.output_count() as usize);
                if curr.output_count() == 0 {
                    continue;
                }

                // Count how many output connections fan out from each port
                // This is so that we don't have to needlessly clone audio buffers
                //
                // If this is inefficient, we can instead maintain this data
                // cached on the node
                output_counts.clear();
                output_counts.resize(curr.output_count() as usize, 0);
                for edge in self.graph.edges(ix) {
                    let edge = edge.weight();
                    for conn in &edge.connections {
                        if let PortIndex::Port(idx) = conn.output_idx {
                            output_counts[idx as usize] += 1;
                        } else {
                            unreachable!()
                        }
                    }
                }

                // all the edges from this node go to nodes which depend on it,
                // i.e. the nodes it outputs to. Store the blocks for retrieval.
                for edge in self.graph.edges(ix) {
                    let edge = edge.weight();
                    for conn in &edge.connections {
                        if let PortIndex::Port(idx) = conn.output_idx {
                            output_counts[idx as usize] -= 1;
                            // if there are no consumers left after this, take the data
                            let block = if output_counts[idx as usize] == 0 {
                                out[conn.output_idx].take()
                            } else {
                                out[conn.output_idx].clone()
                            };
                            *conn.cache.borrow_mut() = Some(block);
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        }
        // The destination node stores its output on itself, extract it.
        self.graph[self.dest_id.0]
            .node
            .borrow_mut()
            .destination_data()
            .expect("Destination node should have data cached")
    }

    /// Obtain a mutable reference to a node
    pub(crate) fn node_mut(&self, ix: NodeId) -> RefMut<'_, Box<dyn AudioNodeEngine>> {
        self.graph[ix.0].node.borrow_mut()
    }
}

impl Node {
    pub fn new(node: Box<dyn AudioNodeEngine>) -> Self {
        Node {
            node: RefCell::new(node),
        }
    }
}

impl Edge {
    pub fn new(input_idx: PortIndex<InputPort>, output_idx: PortIndex<OutputPort>) -> Self {
        Edge {
            connections: SmallVec::from_buf([Connection::new(input_idx, output_idx)]),
        }
    }
}

impl Connection {
    pub fn new(input_idx: PortIndex<InputPort>, output_idx: PortIndex<OutputPort>) -> Self {
        Connection {
            input_idx,
            output_idx,
            cache: RefCell::new(None),
        }
    }
}
