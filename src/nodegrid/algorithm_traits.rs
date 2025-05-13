pub trait Centralized {
    /// Return whether this `Node` is the initiator or not.
    fn initiator(&self) -> bool;
}
