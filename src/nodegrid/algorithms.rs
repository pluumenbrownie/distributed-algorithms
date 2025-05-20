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
use crate::{NodeGrid, SelectedAlgorithm};

mod elections;
mod snapshots;

fn log_sent_messages<T: Display>(messages: &VecDeque<T>, logger: &mut Vec<String>) {
    for mesg in messages.iter() {
        logger.push(format!("Sent {mesg}."));
    }
}

#[derive(Debug, Default)]
struct Algorithm<N, M>
where
    N: NodeLike,
    M: Mesg,
{
    nodes: Vec<N>,
    messages: VecDeque<M>,
}

impl<N, M> Algorithm<N, M>
where
    N: NodeLike,
    M: Mesg,
{
    fn wrap_nodes(nodes: &[Node]) -> Vec<N> {
        let nodes: Vec<_> = nodes.iter().map(N::from).collect();
        nodes
    }

    fn new(nodes: &[Node]) -> Algorithm<N, M> {
        Self {
            nodes: Self::wrap_nodes(nodes),
            ..Default::default()
        }
    }

    fn choose_initiator(&self, logger: &mut Vec<String>) -> String {
        let initiator = self
            .nodes
            .iter()
            .choose(&mut rand::rng())
            .unwrap()
            .name_clone();
        logger.push(format!("Choose {} as initator.", initiator));
        initiator
    }

    fn choose_initiator_multiple(&self, amount: usize, logger: &mut Vec<String>) -> Vec<String> {
        let initiators: Vec<String> = self
            .nodes
            .iter()
            .choose_multiple(&mut rand::rng(), amount)
            .iter()
            .map(|&n| n.name_clone())
            .collect();
        logger.push(format!("Choose {:?} as initator.", initiators));
        initiators
    }

    fn random_node(&mut self) -> &mut N {
        self.nodes.iter_mut().choose(&mut rand::rng()).unwrap()
    }

    fn node_by_name(&mut self, name: String) -> &mut N {
        self.nodes.iter_mut().find(|n| n.name() == name).unwrap()
    }

    fn pop_mesg(&mut self) -> Option<M> {
        self.messages.pop_front()
    }

    fn has_messages(&self) -> bool {
        !self.messages.is_empty()
    }
}

trait Mesg: Clone + Default + Display {}

/// Messages through a channel are received by a node in the same order as they
/// were sent.
trait Fifo: Mesg {}
/// Messages can be received in any order, irrespective of the sending order.
trait NonFifo: Mesg {}

trait FifoChannels<M: Fifo> {
    fn add_mesg(&mut self, mesg: M);
    fn add_mesg_iter(&mut self, messages: &mut VecDeque<M>);
}
impl<M, N> FifoChannels<M> for Algorithm<N, M>
where
    N: NodeLike,
    M: Fifo,
{
    /// Add a FIFO message to the back of the queue.
    fn add_mesg(&mut self, mesg: M) {
        self.messages.push_back(mesg);
    }
    fn add_mesg_iter(&mut self, messages: &mut VecDeque<M>) {
        self.messages.append(messages);
    }
}

trait RandomChannels<M: NonFifo> {
    fn add_mesg(&mut self, mesg: M);
    fn add_mesg_iter(&mut self, messages: &mut VecDeque<M>);
}
impl<M, N> RandomChannels<M> for Algorithm<N, M>
where
    N: NodeLike,
    M: NonFifo,
{
    /// Add a message in a random index of the message queue.
    fn add_mesg(&mut self, mesg: M) {
        if self.has_messages() {
            let index = random_range(0..self.messages.len());
            self.messages.insert(index, mesg);
        } else {
            self.messages.push_back(mesg);
        }
    }
    fn add_mesg_iter(&mut self, messages: &mut VecDeque<M>) {
        for mesg in messages.drain(..) {
            self.add_mesg(mesg);
        }
    }
}

trait NodeLike: for<'a> From<&'a Node> + Default {
    fn name(&self) -> &str;
    fn name_clone(&self) -> String {
        self.name().to_string()
    }
}

#[derive(Debug, Clone, Default)]
struct MessageVec<T>(Vec<T>)
where
    T: Mesg;

impl<T> From<&Vec<T>> for MessageVec<T>
where
    T: Mesg,
{
    fn from(vec: &Vec<T>) -> Self {
        Self(vec.clone())
    }
}

impl<T: Mesg> Deref for MessageVec<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Mesg> DerefMut for MessageVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Display for MessageVec<T>
where
    T: Mesg,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "[]")
        } else {
            writeln!(f, "[")?;
            for i in self.0.iter() {
                writeln!(f, "    {i}")?;
            }
            write!(f, "]")
        }
    }
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[displaydoc("LC({0})")]
struct LamportsClock(usize);
impl LamportsClock {
    fn tick(&mut self) -> Self {
        self.0 += 1;
        *self
    }

    fn receive<T: LamportsMessage>(&mut self, mesg: &T) -> Self {
        self.0 = self.max(&mut mesg.time()).0 + 1;
        *self
    }
}

trait LamportsMessage: Mesg {
    fn time(&self) -> LamportsClock;
}

impl NodeGrid {
    pub fn run_algorithm(
        &mut self,
        algorithm: SelectedAlgorithm,
        logger: &mut Vec<String>,
    ) -> Result<()> {
        let result = match algorithm {
            SelectedAlgorithm::ChandyLamport => self.chandy_lamport(logger),
            SelectedAlgorithm::LaiYang => self.lai_yang(logger),
            SelectedAlgorithm::ChangRoberts => self.chang_roberts(logger),
        };
        if result.is_err() {
            logger.push(format!("{} did not complete.", algorithm));
        }

        Ok(())
    }
}
