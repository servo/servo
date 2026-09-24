/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{RefCell, RefMut};
use std::{cmp, fmt, hash};

use log::error;
use malloc_size_of::MallocSizeOf as MallocSizeOfTrait;
use malloc_size_of_derive::MallocSizeOf;
use petgraph::Direction;
use petgraph::algo::tarjan_scc;
use petgraph::graph::DefaultIx;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use petgraph::visit::{DfsPostOrder, EdgeRef, Reversed};
use rustc_hash::FxHashSet;
use smallvec::SmallVec;

use crate::audio_node::{
    AudioNodeEngine, AudioNodeType, BlockInfo, ChannelCountMode, ChannelInterpretation,
};
use crate::block::{Block, Chunk};
use crate::delay_node::{DelayNode, DelayReader, DelayWriter};
use crate::destination_node::DestinationNode;
use crate::listener::AudioListenerNode;
use crate::param::ParamType;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug, MallocSizeOf)]
/// A unique identifier for nodes in the graph. Stable
/// under graph mutation.
pub struct NodeId(#[ignore_malloc_size_of = "External Type"] NodeIndex<DefaultIx>);

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
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug, MallocSizeOf)]
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
    type ParamId: Copy
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + hash::Hash
        + fmt::Debug
        + MallocSizeOfTrait;
    type Listener: Copy
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + hash::Hash
        + fmt::Debug
        + MallocSizeOfTrait;
}

/// An identifier for a port.
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug, MallocSizeOf)]
pub struct PortId<Kind: PortKind>(NodeId, PortIndex<Kind>);

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, MallocSizeOf)]
/// Marker type for denoting that the port is an input port
/// of the node it is connected to
pub struct InputPort;
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, MallocSizeOf)]
/// Marker type for denoting that the port is an output port
/// of the node it is connected to
pub struct OutputPort;

impl PortKind for InputPort {
    type ParamId = ParamType;
    type Listener = ();
}

#[derive(Debug, Hash, PartialOrd, Ord, PartialEq, Eq, Copy, Clone, MallocSizeOf)]
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

/// A cycle breaker consists of a DelayNode disconnected from the audio graph,
/// as well as the NodeIds of its internal DelayWriter and DelayReader respectively.
/// The internal nodes are currently connected to the graph.
type CycleBreaker = (Box<DelayNode>, NodeId, NodeId);

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

    /// Removes a node and returns it
    pub(crate) fn remove_node(&mut self, node: NodeId) -> Option<Box<dyn AudioNodeEngine>> {
        let maybe_weight = self.graph.remove_node(node.0);
        maybe_weight.map(|weight| weight.node.into_inner())
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
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect>
    pub fn disconnect_all_from(&mut self, node: NodeId) {
        let edges = self.graph.edges(node.0).map(|e| e.id()).collect::<Vec<_>>();
        for edge in edges {
            self.graph.remove_edge(edge);
        }
    }

    /// Disconnect all outgoing connections from a node's output
    ///
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-output>
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
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode>
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
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output>
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
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationparam>
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
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationnode-output-input>
    /// <https://webaudio.github.io/web-audio-api/#dom-audionode-disconnect-destinationparam-output>
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
    ///
    /// This implements steps 4.2 and parts of 4.4 from
    /// <https://webaudio.github.io/web-audio-api/#rendering-loop>
    pub fn process(&mut self, info: &BlockInfo) -> Chunk {
        // Step 4.2. Order the AudioNodes of the BaseAudioContext to be processed.
        let (cycle_nodes, cycle_breakers) = self.detect_nodes_in_cycles();

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

                if cycle_nodes.contains(&ix) {
                    // Step 4.2.7. If nodes contains cycles, mute all the AudioNodes
                    // that are part of this cycle, and remove them from nodes.
                    let chunk = curr.as_mut().mute_node();
                    if curr.output_count() == 0 {
                        // Step 4.4.5. If this AudioNode is a destination node, record
                        // the input of this AudioNode.
                        curr.process(chunk, info);
                    } else {
                        // Step 4.4.6. Else, process the input buffer, and make
                        // available for reading the resulting buffer.
                        self.fill_node_cache_with_silence(ix, curr.output_count());
                    }
                    continue;
                }

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
        // Now that we've finished processing, reset the cycle breakers
        cycle_breakers
            .into_iter()
            .for_each(|cycle_breaker| self.reinsert_cycle_breaker(cycle_breaker));
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

    /// Detect cycles in this graph and return the indices of their nodes.
    ///
    /// This covers the cycle-related part of step 4.2. Tarjan's algorithm finds
    /// the cycles, and [`Self::process`] mutes the affected nodes as required by
    /// step 4.2.7.
    ///
    /// The outer render-loop steps, including returning `render_result`, are
    /// outside this method because [`Self::process`] returns audio data.
    ///
    /// <https://webaudio.github.io/web-audio-api/#rendering-loop>
    ///
    /// > 4. Process a render quantum.
    /// >    2. Order the AudioNodes of the BaseAudioContext to be processed.
    /// >       4. Let cycle breakers be an empty set of DelayNodes. It will contain all the
    /// >          DelayNodes that are part of a cycle.
    /// >       5. For each AudioNode node in nodes:
    /// >          1. If node is a DelayNode that is part of a cycle, add it to cycle breakers
    /// >             and remove it from nodes.
    /// >       6. For each DelayNode delay in cycle breakers:
    /// >          1. Let delayWriter and delayReader respectively be a DelayWriter and a
    /// >             DelayReader, for delay. Add delayWriter and delayReader to nodes. Disconnect
    /// >             delay from all its input and outputs.
    /// >             Note: This breaks the cycle: if a DelayNode is in a cycle, its two ends can be
    /// >             considered separately, because delay lines cannot be smaller than one render
    /// >             quantum when in a cycle.
    /// >       7. If nodes contains cycles, mute all the AudioNodes that are part of this cycle, and
    /// >          remove them from nodes.
    fn detect_nodes_in_cycles(
        &mut self,
    ) -> (
        FxHashSet<NodeIndex<DefaultIx>>,
        Vec<CycleBreaker>,
    ) {
        let mut cycle_nodes = FxHashSet::default();
        let mut cycle_breakers = Vec::default();
        let mut has_cycle_breakers = false;
        for component in tarjan_scc(&self.graph) {
            // Tarjan's algorithm groups the graph into strongly connected components.
            // A component with multiple nodes is a cycle; a single-node component is a
            // cycle only when the node has an edge to itself.
            let is_cycle = component.len() > 1 ||
                component.first().is_some_and(|node_index| {
                    self.graph
                        .edges(*node_index)
                        .any(|edge| edge.target() == *node_index)
                });
            if is_cycle {
                // See if there is a delay node. If there is, break the cycle.
                let delay_nodes = component
                    .iter()
                    .filter_map(|node_index| {
                        if matches!(
                            self.graph[*node_index].node.borrow().node_type(),
                            AudioNodeType::DelayNode
                        ) {
                            self.break_cycle(*node_index)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<CycleBreaker>>();
                if !delay_nodes.is_empty() {
                    has_cycle_breakers = true;
                    cycle_breakers.extend(delay_nodes);
                } else {
                    cycle_nodes.extend(component);
                }
            }
        }
        // If we had cycle breakers, we need to run Tarjan's again.
        // We have only broken cycles with delay nodes.
        // The SCC could have had another cycle with strictly non-delay nodes.
        if has_cycle_breakers {
            cycle_nodes.clear();
            for component in tarjan_scc(&self.graph) {
                let is_cycle = component.len() > 1 ||
                    component.first().is_some_and(|node_index| {
                        self.graph
                            .edges(*node_index)
                            .any(|edge| edge.target() == *node_index)
                    });
                if is_cycle {
                    cycle_nodes.extend(component);
                }
            }
        }
        (cycle_nodes, cycle_breakers)
    }

    /// <https://webaudio.github.io/web-audio-api/#available-for-reading>
    ///
    /// Making a buffer available for reading from an AudioNode means putting
    /// it in a state where other AudioNodes connected to this AudioNode can
    /// safely read from it.
    ///
    /// The specification does not require these buffers to be silent. This helper
    /// runs after [`AudioNodeEngine::mute_node`], so it fills the caches with
    /// silence for downstream nodes.
    fn fill_node_cache_with_silence(&self, node_index: NodeIndex<DefaultIx>, output_count: u32) {
        for edge in self.graph.edges(node_index) {
            let edge = edge.weight();
            for connection in &edge.connections {
                if let PortIndex::Port(index) = connection.output_idx {
                    if index < output_count {
                        *connection.cache.borrow_mut() = Some(Block::default());
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }

    /// Disconnects all incoming edges to a node.
    /// Stores the edge id and node index.
    fn disconnect_incoming_connections(
        &mut self,
        node_index: NodeIndex<DefaultIx>,
    ) -> Vec<(NodeIndex, Edge)> {
        let incoming_node_indices = self
            .graph
            .edges_directed(node_index, Direction::Incoming)
            .map(|edge| (edge.id(), edge.source()))
            .collect::<Vec<_>>();
        incoming_node_indices
            .into_iter()
            .map(|(edge_id, incoming_node_index)| {
                (
                    incoming_node_index,
                    self.graph
                        .remove_edge(edge_id)
                        .expect("Edge came from graph iterator, must exist"),
                )
            })
            .collect::<Vec<_>>()
    }

    /// Disconnects all outgoing edges from a node.
    /// Stores the edge id and node index.
    fn disconnect_outgoing_connections(
        &mut self,
        node_index: NodeIndex<DefaultIx>,
    ) -> Vec<(NodeIndex, Edge)> {
        let outgoing_node_indices = self
            .graph
            .edges(node_index)
            .map(|edge| (edge.id(), edge.target()))
            .collect::<Vec<_>>();
        outgoing_node_indices
            .into_iter()
            .map(|(edge_id, outgoing_node_index)| {
                (
                    outgoing_node_index,
                    self.graph
                        .remove_edge(edge_id)
                        .expect("Edge came from graph iterator, must exist"),
                )
            })
            .collect::<Vec<_>>()
    }

    /// Breaks cycles with a [DelayNode] by removing it from the graph, and then rerouting
    /// connections to its internal [DelayReader] and [DelayWriter].
    /// See detect_nodes_in_cycles for a detailed description of the cycle breaker specs.
    ///
    /// For a given delay node, there is only one input and one output.
    /// We connect the input to the DelayWriter, and connect the DelayReader to the output.
    /// 1.18
    /// > This interface is an AudioNode with a single input and single output.
    ///
    /// 1.18.4
    /// > [DelayWriter] has the same input connections as the [DelayNode] it was created from.
    /// > [DelayReader] is connected to the same AudioNodes as the [DelayNode] it was created from.
    fn break_cycle(&mut self, delay_node_index: NodeIndex) -> Option<CycleBreaker> {
        // Disconnect the delay node and save its neighbors.
        let incoming_nodes = self.disconnect_incoming_connections(delay_node_index);
        let outgoing_nodes = self.disconnect_outgoing_connections(delay_node_index);
        // Remove the delay node from the graph.
        let Some(delay_node) = self.remove_node(NodeId(delay_node_index)) else {
            error!("Cycle breaker node index not in the graph");
            return None;
        };
        // Downcast the node to a DelayNode. Only delay nodes are cycle breakers.
        let Ok(mut delay_node) = delay_node.into_any().downcast::<DelayNode>() else {
            error!("Node at cycle breaker node index is not a DelayNode");
            return None;
        };
        // Temporarily remove ownership of DelayNode's internal DelayReader and DelayWriter,
        // and pass ownership to graph. Will regain ownership after processing completes.
        let Some(delay_writer) = delay_node.take_delay_writer() else {
            error!("Tried to break cycle without a DelayWriter");
            return None;
        };
        let Some(mut delay_reader) = delay_node.take_delay_reader() else {
            error!("Tried to break cycle without a DelayReader");
            return None;
        };
        // Set the delay reader cycle breaker status so that it sets the correct minimum delay time.
        delay_reader.set_cycle_breaker_status(true);
        // Add the DelayWriter and DelayReader to the graph.
        let delay_writer_id = self.add_node(delay_writer);
        let delay_reader_id = self.add_node(delay_reader);
        // The DelayWriter is a destination node.
        self.add_extra_dest(delay_writer_id);
        // Reroute delay node's neighbors to the DelayReader and DelayWriter.
        incoming_nodes.into_iter().for_each(|(input, weight)| {
            self.graph.add_edge(input, delay_writer_id.0, weight);
        });
        outgoing_nodes.into_iter().for_each(|(output, weight)| {
            self.graph.add_edge(delay_reader_id.0, output, weight);
        });
        Some((delay_node, delay_writer_id, delay_reader_id))
    }

    /// Reinserts cycle breakers into the audio graph after processing the render quantum loop.
    /// We do this after every processing loop because connections can change each iteration.
    fn reinsert_cycle_breaker(&mut self, cycle_breaker: CycleBreaker) {
        let (mut delay_node, delay_writer_id, delay_reader_id) = cycle_breaker;
        // Disconnect the connections to the DelayWriter and out of the DelayReader.
        let incoming_nodes = self.disconnect_incoming_connections(delay_writer_id.0);
        let outgoing_nodes = self.disconnect_outgoing_connections(delay_reader_id.0);
        // Remove the DelayReader and DelayWriter from the graph.
        let Some(delay_writer) = self.remove_node(delay_writer_id) else {
            error!("Tried to remove a non-existent DelayWriter");
            return;
        };
        let Ok(delay_writer) = delay_writer.into_any().downcast::<DelayWriter>() else {
            error!("Failed to downcast graph node to DelayWriter");
            return;
        };
        let Some(delay_reader) = self.remove_node(delay_reader_id) else {
            error!("Tried to remove a non-existent DelayReader");
            return;
        };
        let Ok(delay_reader) = delay_reader.into_any().downcast::<DelayReader>() else {
            error!("Failed to downcast graph node to DelayReader");
            return;
        };
        // DelayNode regains ownership of its internal DelayReader and DelayWriter.
        // This resets the cycle breaker status.
        delay_node.set_delay_writer(Some(delay_writer));
        delay_node.set_delay_reader(Some(delay_reader));
        delay_node.set_cycle_breaker_status(false);
        // Insert the delay node back into the graph.
        let delay_node_id = self.add_node(delay_node);
        // Reroute the connections from the DelayReader and DelayWriter back to the parent DelayNode
        incoming_nodes.into_iter().for_each(|(input, weight)| {
            self.graph.add_edge(input, delay_node_id.0, weight);
        });
        outgoing_nodes.into_iter().for_each(|(output, weight)| {
            self.graph.add_edge(delay_node_id.0, output, weight);
        });
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
