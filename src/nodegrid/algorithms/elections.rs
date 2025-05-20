use anyhow::{Ok, Result, anyhow};
use displaydoc::Display;
use rand::{
    random_range,
    seq::{IndexedRandom, IteratorRandom},
};
use std::{
    collections::VecDeque,
    fmt::{Display, format},
    ops::{Deref, DerefMut},
};

use crate::node::{Node, connection};
use crate::{NodeGrid, SelectedAlgorithm, nodegrid::algorithms::*};

mod changroberts {
    use anyhow::{Ok, Result, anyhow};
    use displaydoc::Display;
    use rand::seq::{IndexedRandom, IteratorRandom};
    use std::{
        collections::VecDeque,
        default,
        fmt::{Debug, format},
        path::Display,
    };
    use strum::EnumIs;

    use crate::{
        node::{Node, connection},
        nodegrid::{NodeGrid, SelectedAlgorithm, algorithms::snapshots::*, algorithms::*},
    };

    #[derive(Debug, Default, Clone)]
    struct AlgNode {
        node: Node,
        state: NodeState,
    }

    #[derive(Debug, Default, Clone, EnumIs)]
    enum NodeState {
        #[default]
        Active,
        Passive,
        Leader,
    }

    #[derive(Debug, Display, Default, Clone)]
    #[displaydoc("<{kind}={id}> {sender}->{destination}")]
    struct Message {
        sender: String,
        destination: String,
        id: usize,
        kind: MesgKind,
    }
    impl Mesg for Message {}
    impl NonFifo for Message {}

    #[derive(Debug, Display, Default, Clone, Copy, PartialEq, Eq, EnumIs)]
    enum MesgKind {
        #[default]
        /// leader
        Leader,
    }

    impl From<&Node> for AlgNode {
        fn from(node: &Node) -> Self {
            AlgNode {
                node: node.clone(),
                ..Default::default()
            }
        }
    }
    impl NodeLike for AlgNode {
        fn name(&self) -> &str {
            &self.node.name
        }
    }

    impl Message {
        fn new(sender: String, destination: String, id: usize) -> Self {
            Message {
                sender,
                destination,
                id,
                ..Default::default()
            }
        }

        fn pass_on(self, sender: String, receiver: String) -> Message {
            Message {
                sender,
                destination: receiver,
                id: self.id,
                kind: self.kind,
            }
        }
    }

    impl AlgNode {
        fn handle_message(&mut self, mesg: Message, logger: &mut Vec<String>) -> VecDeque<Message> {
            logger.push(format!("{}={} received {mesg}", self.name(), {
                if self.state.is_passive() {
                    "passive".to_string()
                } else {
                    self.node.id.to_string()
                }
            }));

            let mut output: VecDeque<Message> = VecDeque::new();

            match self.state {
                NodeState::Passive => {
                    let receiver = self.node.connections[0].other.clone();
                    output.push_back(mesg.pass_on(self.name_clone(), receiver))
                }
                NodeState::Active => {
                    let ordering = mesg.id.cmp(&self.node.id);
                    // q <=> p
                    match ordering {
                        std::cmp::Ordering::Less => {
                            logger.push(format!(
                                "{}<{} so the message is dismissed.",
                                mesg.id, self.node.id
                            ));
                        }
                        std::cmp::Ordering::Greater => {
                            logger.push(format!(
                                "{}>{} so {} is now passive.",
                                mesg.id,
                                self.node.id,
                                self.name()
                            ));
                            self.state = NodeState::Passive;
                            let receiver = self.node.connections[0].other.clone();
                            output.push_back(mesg.pass_on(self.name_clone(), receiver));
                        }
                        std::cmp::Ordering::Equal => {
                            logger.push(format!(
                                "{}={} so {} declares itself the leader.",
                                mesg.id,
                                self.node.id,
                                self.name()
                            ));
                            self.state = NodeState::Leader;
                        }
                    }
                }
                NodeState::Leader => {}
            }

            output
        }

        fn initiate(&self) -> Message {
            let destination = self.node.connections[0].other.clone();
            Message::new(self.name_clone(), destination, self.node.id)
        }
    }

    impl Algorithm<AlgNode, Message> {
        fn run(&mut self, logger: &mut Vec<String>) -> Result<()> {
            let initiators: Vec<String> = self.nodes.iter().map(|n| n.name_clone()).collect();
            for node_name in initiators.into_iter() {
                let init_node = self.node_by_name(node_name);
                let mesg = init_node.initiate();
                self.add_mesg(mesg);
            }

            while !self.messages.is_empty() && !self.nodes.iter().any(|n| n.state.is_leader()) {
                let mesg = self.pop_mesg().unwrap();
                let mut response = self
                    .node_by_name(mesg.destination.clone())
                    .handle_message(mesg, logger);
                self.add_mesg_iter(&mut response);
            }

            if let Some(leader) = self.nodes.iter().find(|n| n.state.is_leader()) {
                logger.push(format!("Node {} was chosen as leader.", leader.name()));
            } else {
                logger.push("Leader election failed.".to_string());
            }
            Ok(())
        }
    }

    impl NodeGrid {
        pub fn chang_roberts(&mut self, logger: &mut Vec<String>) -> Result<()> {
            self.check_not_empty(logger)?;
            let mut algorithm: Algorithm<AlgNode, Message> = Algorithm::new(&self.nodes);
            logger.push(format!(
                "Started Chang-Roberts election with {} nodes.",
                algorithm.nodes.len()
            ));
            algorithm.run(logger)
        }
    }
}
