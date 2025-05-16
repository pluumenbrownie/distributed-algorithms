use anyhow::{Ok, Result, anyhow};
use displaydoc::Display;
use rand::seq::{IndexedRandom, IteratorRandom};
use std::{collections::VecDeque, fmt::format};

use crate::node::{Node, connection};

use super::{Algorithm, NodeGrid};

#[derive(Debug, Default, Clone)]
struct ChandyLamportNode {
    node: Node,
    state: isize,
    received: Vec<String>,
    snapshot: Option<Snapshot>,
}

#[derive(Debug, Display, Default, Clone)]
#[displaydoc("Snapshot({state}, {messages:?})")]
struct Snapshot {
    state: isize,
    messages: Vec<ChandyLamportMessage>,
}

impl Snapshot {
    fn new(state: isize) -> Self {
        Snapshot {
            state,
            ..Default::default()
        }
    }
}

impl ChandyLamportNode {
    fn new(node: &Node) -> ChandyLamportNode {
        ChandyLamportNode {
            node: node.clone(),
            ..Default::default()
        }
    }

    fn create_snapshot(&mut self, logger: &mut Vec<String>) -> VecDeque<ChandyLamportMessage> {
        self.snapshot = Some(Snapshot::new(self.state));
        logger.push(format!(
            "{} took {}",
            self.node.name,
            self.snapshot.as_ref().unwrap()
        ));

        let mut outgoing = VecDeque::new();

        for connection in self.node.connections.iter() {
            outgoing.push_back(ChandyLamportMessage {
                sender: self.node.name.clone(),
                destination: connection.other.clone(),
                kind: ChLaMesgKind::Mark,
            });
        }
        log_sent_messages(&outgoing, logger);

        outgoing
    }

    fn handle_message(
        &mut self,
        mesg: ChandyLamportMessage,
        logger: &mut Vec<String>,
    ) -> VecDeque<ChandyLamportMessage> {
        logger.push(format!("{} received {mesg}", self.node.name));

        match mesg.kind {
            ChLaMesgKind::Mark => {
                let mut output = VecDeque::new();
                if self.snapshot.is_none() {
                    output = self.create_snapshot(logger);
                }

                self.received.push(mesg.sender.clone());
                logger.push(format!(
                    "{} notes it has received <mark> from {}.",
                    self.node.name, mesg.sender
                ));
                output
            }
            ChLaMesgKind::Increment => {
                self.update_snapshot(mesg, logger);
                self.state += 1;
                VecDeque::new()
            }
            ChLaMesgKind::Decrement => {
                self.update_snapshot(mesg, logger);
                self.state -= 1;
                VecDeque::new()
            }
        }
    }

    fn update_snapshot(&mut self, mesg: ChandyLamportMessage, logger: &mut Vec<String>) {
        if let Some(snapshot) = &mut self.snapshot {
            if !self.received.contains(&mesg.sender) {
                logger.push(format!("{} saves {mesg} in snapshot.", self.node.name));
                snapshot.messages.push(mesg);
            }
        }
    }

    fn random_process(&mut self, logger: &mut Vec<String>) -> ChandyLamportMessage {
        let destination = self
            .node
            .connections
            .iter()
            .choose(&mut rand::rng())
            .expect("Node has no connections.");
        let mesg = ChandyLamportMessage::random(self.node.name.clone(), destination.other.clone());
        match mesg.kind {
            ChLaMesgKind::Decrement => {
                self.state += 1;
                logger.push(format!("{}={} and send {mesg}", self.node.name, self.state));
            }
            ChLaMesgKind::Increment => {
                self.state -= 1;
                logger.push(format!("{}={} and send {mesg}", self.node.name, self.state));
            }
            _ => {}
        };
        mesg
    }
}

#[derive(Debug, Display, Default, Clone)]
#[displaydoc("<{kind}> {sender}->{destination}")]
struct ChandyLamportMessage {
    sender: String,
    destination: String,
    kind: ChLaMesgKind,
}

impl ChandyLamportMessage {
    fn random(sender: String, destination: String) -> Self {
        Self {
            sender,
            destination,
            kind: [ChLaMesgKind::Increment, ChLaMesgKind::Decrement]
                .choose(&mut rand::rng())
                .unwrap()
                .to_owned(),
        }
    }
}

#[derive(Debug, Display, Default, Clone, PartialEq, Eq)]
enum ChLaMesgKind {
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
        if self.nodes.is_empty() {
            logger.push("No nodes in grid.".to_string());
            return Err(anyhow!("No nodes in grid."));
        }

        let mut nodes = self.wrap_nodes(logger);
        let mut messages = VecDeque::new();

        let initiator = choose_initiator(logger, &nodes);

        for i in 0..5 {
            let node = nodes.iter_mut().choose(&mut rand::rng()).unwrap();
            messages.push_back(node.random_process(logger));
        }

        // while !messages.is_empty() {
        //     let mesg = messages.pop_front().unwrap();
        //     let mut response =
        //         node_by_name(&mut nodes, mesg.destination.clone()).handle_message(mesg, logger);
        //     messages.append(&mut response);
        // }

        let mut response = node_by_name(&mut nodes, initiator).create_snapshot(logger);
        messages.append(&mut response);

        for i in 0..5 {
            let node = nodes.iter_mut().choose(&mut rand::rng()).unwrap();
            let mesg = node.random_process(logger);
            messages.push_back(mesg);
        }

        while !messages.is_empty() {
            let mesg = messages.pop_front().unwrap();
            let mut response =
                node_by_name(&mut nodes, mesg.destination.clone()).handle_message(mesg, logger);
            if !response.is_empty() {
                messages.append(&mut response);
                for i in 0..3 {
                    let node = nodes.iter_mut().choose(&mut rand::rng()).unwrap();
                    messages.push_back(node.random_process(logger));
                }
            }
        }

        verify_snapshot(logger, nodes);

        Ok(())
    }

    fn wrap_nodes(&mut self, logger: &mut Vec<String>) -> Vec<ChandyLamportNode> {
        let nodes: Vec<_> = self.nodes.iter().map(ChandyLamportNode::new).collect();
        logger.push(format!(
            "Started Chandy-Lampart snapshot with {} nodes.",
            nodes.len()
        ));
        nodes
    }

    pub(crate) fn run_algorithm(
        &mut self,
        algorithm: Algorithm,
        logger: &mut Vec<String>,
    ) -> Result<()> {
        let result = match algorithm {
            Algorithm::ChandyLamport => self.chandy_lamport(logger),
            Algorithm::LaiYang => todo!("To early bro."),
        };
        if result.is_err() {
            logger.push(format!("{} did not complete.", algorithm));
        }

        Ok(())
    }
}

fn verify_snapshot(logger: &mut Vec<String>, nodes: Vec<ChandyLamportNode>) {
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
                .filter(|m| m.kind == ChLaMesgKind::Increment)
                .count() as isize
                - n.snapshot
                    .as_ref()
                    .unwrap()
                    .messages
                    .iter()
                    .filter(|m| m.kind == ChLaMesgKind::Decrement)
                    .count() as isize
        });
        logger.push(format!("Node total: {snapshot_sum_of_states}"));
        logger.push(format!("Message total: {snapshot_sum_of_messages}"));
    } else {
        logger.push("Snapshot did not complete.".to_string());
    }
    // for node in nodes.iter() {
    //     logger.push(format!("{:?}", node.snapshot));
    // }
    logger.push(String::new());
}

fn log_sent_messages(messages: &VecDeque<ChandyLamportMessage>, logger: &mut Vec<String>) {
    for mesg in messages.iter() {
        logger.push(format!("Sent {mesg}."));
    }
}

fn node_by_name(nodes: &mut [ChandyLamportNode], name: String) -> &mut ChandyLamportNode {
    nodes.iter_mut().find(|n| n.node.name == name).unwrap()
}

fn choose_initiator(logger: &mut Vec<String>, nodes: &[ChandyLamportNode]) -> String {
    let initiator = nodes
        .iter()
        .choose(&mut rand::rng())
        .unwrap()
        .node
        .name
        .clone();
    logger.push(format!("Choose {} as initator.", initiator));
    initiator
}

// impl ChandyLamportNode {
//     fn receive(&self, message) ->
// }
