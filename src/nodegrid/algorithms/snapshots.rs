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

#[derive(Debug, Display, Default, Clone)]
struct Snapshot<T: Mesg> {
    state: isize,
    timestamp: LamportsClock,
    messages: MessageVec<T>,
}

impl<T: Mesg> Snapshot<T> {
    fn new(state: isize) -> Self {
        Snapshot {
            state,
            ..Default::default()
        }
    }
}

impl<T: Mesg> std::fmt::Display for Snapshot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Snapshot({}, {})", self.state, &self.messages)
    }
}

mod chandylamport {
    use anyhow::{Ok, Result, anyhow};
    use displaydoc::Display;
    use rand::seq::{IndexedRandom, IteratorRandom};
    use std::{
        collections::VecDeque,
        default,
        fmt::{Debug, format},
        path::Display,
    };

    use crate::{
        node::{Node, connection},
        nodegrid::{NodeGrid, SelectedAlgorithm, algorithms::snapshots::*, algorithms::*},
    };

    #[derive(Debug, Default, Clone)]
    struct AlgNode {
        node: Node,
        state: isize,
        received: Vec<String>,
        snapshot: Option<Snapshot<Message>>,
    }

    impl AlgNode {
        fn create_snapshot(&mut self, logger: &mut Vec<String>) -> VecDeque<Message> {
            self.snapshot = Some(Snapshot::new(self.state));
            logger.push(format!(
                "{} took {}",
                self.name(),
                self.snapshot.as_ref().unwrap()
            ));

            let mut outgoing = VecDeque::new();

            for connection in self.node.connections.iter() {
                outgoing.push_back(Message {
                    sender: self.name_clone(),
                    destination: connection.other.clone(),
                    kind: MesgKind::Mark,
                });
            }
            log_sent_messages(&outgoing, logger);

            outgoing
        }

        fn handle_message(&mut self, mesg: Message, logger: &mut Vec<String>) -> VecDeque<Message> {
            logger.push(format!("{} received {mesg}", self.name()));

            match mesg.kind {
                MesgKind::Mark => {
                    let mut output = VecDeque::new();
                    if self.snapshot.is_none() {
                        output = self.create_snapshot(logger);
                    }

                    self.received.push(mesg.sender.clone());
                    logger.push(format!(
                        "{} notes it has received <mark> from {}.",
                        self.name(),
                        mesg.sender
                    ));
                    output
                }
                MesgKind::Increment => {
                    self.update_snapshot(mesg, logger);
                    self.state += 1;
                    VecDeque::new()
                }
                MesgKind::Decrement => {
                    self.update_snapshot(mesg, logger);
                    self.state -= 1;
                    VecDeque::new()
                }
            }
        }

        fn update_snapshot(&mut self, mesg: Message, logger: &mut Vec<String>) {
            if let Some(snapshot) = &mut self.snapshot {
                if !self.received.contains(&mesg.sender) {
                    logger.push(format!("{} saves {mesg} in snapshot.", self.node.name));
                    snapshot.messages.push(mesg);
                }
            }
        }

        fn random_process(&mut self, logger: &mut Vec<String>) -> Message {
            let destination = self
                .node
                .connections
                .iter()
                .choose(&mut rand::rng())
                .expect("Node has no connections.");
            let mesg = Message::random(self.name_clone(), destination.other.clone());
            match mesg.kind {
                MesgKind::Decrement => {
                    self.state += 1;
                    logger.push(format!("{}={} and send {mesg}", self.name(), self.state));
                }
                MesgKind::Increment => {
                    self.state -= 1;
                    logger.push(format!("{}={} and send {mesg}", self.name(), self.state));
                }
                _ => {}
            };
            mesg
        }
    }

    #[derive(Debug, Display, Default, Clone)]
    #[displaydoc("<{kind}> {sender}->{destination}")]
    struct Message {
        sender: String,
        destination: String,
        kind: MesgKind,
    }
    impl Mesg for Message {}
    impl Fifo for Message {}

    impl Message {
        fn random(sender: String, destination: String) -> Self {
            Self {
                sender,
                destination,
                kind: [MesgKind::Increment, MesgKind::Decrement]
                    .choose(&mut rand::rng())
                    .unwrap()
                    .to_owned(),
            }
        }
    }

    #[derive(Debug, Display, Default, Clone, PartialEq, Eq)]
    enum MesgKind {
        #[default]
        /// mark
        Mark,
        /// increment
        Increment,
        /// decrement
        Decrement,
    }

    impl NodeGrid {
        pub fn chandy_lamport(&mut self, logger: &mut Vec<String>) -> Result<()> {
            self.check_not_empty(logger)?;
            let mut algorithm: Algorithm<AlgNode, Message> = Algorithm::new(&self.nodes);
            logger.push(format!(
                "Started Chandy-Lamport snapshot with {} nodes.",
                algorithm.nodes.len()
            ));
            algorithm.run(logger)
        }
    }

    impl Algorithm<AlgNode, Message> {
        fn run(&mut self, logger: &mut Vec<String>) -> Result<()> {
            let initiator = self.choose_initiator(logger);

            for i in 0..5 {
                let node = self.random_node();
                let mesg = node.random_process(logger);
                self.add_mesg(mesg);
            }

            let mut response = self.node_by_name(initiator).create_snapshot(logger);
            self.add_mesg_iter(&mut response);

            for i in 0..5 {
                let node = self.random_node();
                let mesg = node.random_process(logger);
                self.add_mesg(mesg);
            }

            while self.has_messages() {
                let mesg = self.pop_mesg().unwrap();
                let mut response = self
                    .node_by_name(mesg.destination.clone())
                    .handle_message(mesg, logger);
                if !response.is_empty() {
                    self.add_mesg_iter(&mut response);
                    for i in 0..3 {
                        let node = self.random_node();
                        let mesg = node.random_process(logger);
                        self.add_mesg(mesg);
                    }
                }
            }

            verify_snapshot(logger, self);

            Ok(())
        }
    }

    fn verify_snapshot(logger: &mut Vec<String>, algorithm: &Algorithm<AlgNode, Message>) {
        let nodes = &algorithm.nodes;
        logger.push(String::new());
        if nodes.iter().all(|n| n.snapshot.is_some()) {
            logger.push("Snapshot completed.".to_string());
            let snapshot_sum_of_states = nodes
                .iter()
                .fold(0isize, |acc, n| acc + n.snapshot.as_ref().unwrap().state);
            let snapshot_sum_of_messages = nodes.iter().fold(0isize, |acc, n| {
                acc + n
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .messages
                    .iter()
                    .filter(|m| m.kind == MesgKind::Increment)
                    .count() as isize
                    - n.snapshot
                        .as_ref()
                        .unwrap()
                        .messages
                        .iter()
                        .filter(|m| m.kind == MesgKind::Decrement)
                        .count() as isize
            });
            logger.push(format!("Node total: {snapshot_sum_of_states}"));
            logger.push(format!("Message total: {snapshot_sum_of_messages}"));
        } else {
            logger.push("Snapshot did not complete.".to_string());
        }
        for node in nodes.iter() {
            logger.push(format!(
                "{:?}",
                node.snapshot
                    .as_ref()
                    .map(Snapshot::to_string)
                    .unwrap_or("None".to_string())
            ));
        }
        logger.push(String::new());
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
}

mod laiyang {
    use anyhow::{Ok, Result, anyhow};
    use displaydoc::Display;
    use rand::seq::{IndexedRandom, IteratorRandom};
    use std::{
        collections::{HashMap, VecDeque},
        default,
        fmt::format,
        u8,
    };

    use crate::{
        node::{Node, connection},
        nodegrid::{NodeGrid, SelectedAlgorithm, algorithms::snapshots::*, algorithms::*},
    };

    #[derive(Debug, Default, Clone)]
    struct AlgNode {
        node: Node,
        state: isize,
        mesg_received: HashMap<String, u8>,
        mesg_sent: HashMap<String, u8>,
        mesg_pre_snapshot: HashMap<String, u8>,
        snapshot: Option<Snapshot<Message>>,
        done: bool,
    }
    impl AlgNode {
        fn create_snapshot(&mut self, logger: &mut Vec<String>) -> VecDeque<Message> {
            self.snapshot = Some(Snapshot::new(self.state));
            logger.push(format!(
                "{} took {}",
                self.name(),
                self.snapshot.as_ref().unwrap()
            ));

            let mut outgoing = VecDeque::new();
            self.send_marks(&mut outgoing);
            log_sent_messages(&outgoing, logger);

            outgoing
        }

        fn handle_message(&mut self, mesg: Message, logger: &mut Vec<String>) -> VecDeque<Message> {
            logger.push(format!("{} received {mesg}", self.name()));
            let sender = mesg.sender;

            self.mesg_received
                .entry(sender)
                .and_modify(|c| *c += 1)
                .or_insert(1);

            //     let output = match mesg.kind {
            //         MesgKind::Mark(mesg_count) => {
            //             let mut output = VecDeque::new();
            //             if self.snapshot.is_none() {
            //                 output = self.create_snapshot(logger);
            //             }
            //             self.mesg_pre_snapshot.insert(mesg.sender, mesg_count);

            //             logger.push(format!("{} notes it has received {mesg}", self.name(),));
            //             output
            //         }
            //         MesgKind::Increment(post_snapshot) => {
            //             let mut output = VecDeque::new();
            //             if !post_snapshot {
            //                 self.update_snapshot(mesg, logger);
            //             } else {
            //                 if self.snapshot.is_none() {
            //                     logger.push(format!(
            //                         "{} takes a snapshot, because the received message is true.",
            //                         self.name()
            //                     ));
            //                     output = self.create_snapshot(logger);
            //                     self.mesg_pre_snapshot.insert(mesg.sender, u8::MAX);
            //                 }
            //             }
            //             self.state += 1;
            //             output
            //         }
            //         MesgKind::Decrement(post_snapshot) => {
            //             let mut output = VecDeque::new();
            //             if !post_snapshot {
            //                 self.update_snapshot(mesg, logger);
            //             } else {
            //                 if self.snapshot.is_none() {
            //                     logger.push(format!(
            //                         "{} takes a snapshot, because the received message is true.",
            //                         self.name()
            //                     ));
            //                     output = self.create_snapshot(logger);
            //                     self.mesg_pre_snapshot.insert(mesg.sender, u8::MAX);
            //                 }
            //             }
            //             self.state -= 1;
            //             output
            //         }
            //     };
            //     if self.snapshot.is_some() && self.mesg_pre_snapshot == self.mesg_received {
            //         self.done = true;
            //     }
            // output
            todo!()
        }

        // fn update_snapshot(&mut self, mesg: Message, logger: &mut Vec<String>) {
        //     if let Some(snapshot) = &mut self.snapshot {
        //         if !self.received.contains(&mesg.sender) {
        //             logger.push(format!("{} saves {mesg} in snapshot.", self.node.name));
        //             snapshot.messages.push(mesg);
        //         }
        //     }
        // }

        fn random_process(&mut self, logger: &mut Vec<String>) -> Message {
            let mesg = self.send_random();
            match mesg.kind {
                MesgKind::Decrement(_) => {
                    self.state += 1;
                    logger.push(format!("{}={} and send {mesg}", self.name(), self.state));
                }
                MesgKind::Increment(_) => {
                    self.state -= 1;
                    logger.push(format!("{}={} and send {mesg}", self.name(), self.state));
                }
                _ => {}
            };
            mesg
        }

        fn send_random(&mut self) -> Message {
            let destination = self
                .node
                .connections
                .iter()
                .choose(&mut rand::rng())
                .expect("Node has no connections.");
            *self.mesg_sent.get_mut(&destination.other).unwrap() += 1;
            Message::random(
                self.name_clone(),
                destination.other.clone(),
                self.snapshot.is_some(),
            )
        }

        fn send_marks(&mut self, outgoing: &mut VecDeque<Message>) {
            for connection in self.node.connections.iter() {
                let destination = connection.other.clone();
                let kind = MesgKind::Mark(*self.mesg_sent.get(&destination).unwrap());
                let mesg = Message {
                    sender: self.name_clone(),
                    destination,
                    kind,
                };
                outgoing.push_back(mesg);
            }
        }
    }

    #[derive(Debug, Display, Default, Clone)]
    #[displaydoc("<{kind}> {sender}->{destination}")]
    struct Message {
        sender: String,
        destination: String,
        kind: MesgKind,
    }
    impl Mesg for Message {}
    impl NonFifo for Message {}

    impl Message {
        fn random(sender: String, destination: String, post_snapshot: bool) -> Self {
            Self {
                sender,
                destination,
                kind: [
                    MesgKind::Increment(post_snapshot),
                    MesgKind::Decrement(post_snapshot),
                ]
                .choose(&mut rand::rng())
                .unwrap()
                .to_owned(),
            }
        }
    }

    #[derive(Debug, Display, Clone, PartialEq, Eq)]
    enum MesgKind {
        /// mark
        Mark(u8),
        /// increment
        Increment(bool),
        /// decrement
        Decrement(bool),
    }

    impl Default for MesgKind {
        fn default() -> Self {
            MesgKind::Mark(0)
        }
    }

    impl NodeGrid {
        pub fn lai_yang(&mut self, logger: &mut Vec<String>) -> Result<()> {
            self.check_not_empty(logger)?;
            let mut algorithm: Algorithm<AlgNode, Message> = Algorithm::new(&self.nodes);
            logger.push(format!(
                "Started Lai-Yang snapshot with {} nodes.",
                algorithm.nodes.len()
            ));
            algorithm.run(logger)
        }
    }
    impl Algorithm<AlgNode, Message> {
        fn run(&mut self, logger: &mut Vec<String>) -> Result<()> {
            let initiator = self.choose_initiator(logger);

            for i in 0..5 {
                let node = self.random_node();
                let mesg = node.random_process(logger);
                self.add_mesg(mesg);
            }

            let mut response = self.node_by_name(initiator).create_snapshot(logger);
            self.add_mesg_iter(&mut response);

            for i in 0..5 {
                let node = self.random_node();
                let mesg = node.random_process(logger);
                self.add_mesg(mesg);
            }

            while self.has_messages() {
                let mesg = self.pop_mesg().unwrap();
                let mut response = self
                    .node_by_name(mesg.destination.clone())
                    .handle_message(mesg, logger);
                if !response.is_empty() {
                    self.add_mesg_iter(&mut response);
                    for i in 0..3 {
                        let node = self.random_node();
                        let mesg = node.random_process(logger);
                        self.add_mesg(mesg);
                    }
                }
            }

            //         verify_snapshot(logger, self);

            Ok(())
        }
    }

    impl From<&Node> for AlgNode {
        fn from(node: &Node) -> Self {
            let mut mesg_sent = HashMap::new();
            for destination in node.connections.iter().map(|c| c.other.clone()) {
                mesg_sent.insert(destination, 0);
            }
            AlgNode {
                node: node.clone(),
                mesg_sent,
                ..Default::default()
            }
        }
    }

    impl NodeLike for AlgNode {
        fn name(&self) -> &str {
            &self.node.name
        }
    }

    fn verify_snapshot(logger: &mut Vec<String>, algorithm: &Algorithm<AlgNode, Message>) {
        let nodes = &algorithm.nodes;
        logger.push(String::new());
        if nodes.iter().all(|n| n.snapshot.is_some()) {
            logger.push("Snapshot completed.".to_string());
            let snapshot_sum_of_states = nodes
                .iter()
                .fold(0isize, |acc, n| acc + n.snapshot.as_ref().unwrap().state);
            let snapshot_sum_of_messages = nodes.iter().fold(0isize, |acc, n| {
                acc + n
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .messages
                    .iter()
                    .filter(|m| {
                        m.kind == MesgKind::Increment(false) || m.kind == MesgKind::Increment(true)
                    })
                    .count() as isize
                    - n.snapshot
                        .as_ref()
                        .unwrap()
                        .messages
                        .iter()
                        .filter(|m| {
                            m.kind == MesgKind::Decrement(false)
                                || m.kind == MesgKind::Decrement(true)
                        })
                        .count() as isize
            });
            logger.push(format!("Node total: {snapshot_sum_of_states}"));
            logger.push(format!("Message total: {snapshot_sum_of_messages}"));
        } else {
            logger.push("Snapshot did not complete.".to_string());
        }
        for node in nodes.iter() {
            logger.push(format!(
                "{:?}",
                node.snapshot
                    .as_ref()
                    .map(Snapshot::to_string)
                    .unwrap_or("None".to_string())
            ));
        }
        logger.push(String::new());
    }
}
