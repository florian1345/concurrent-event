//! Contains the definition of the handler ID which is used for referencing
//! event handlers after registration.

use rand::Rng;

/// The number of random bytes contained in a handler ID. This should be
/// sufficiently high to resist random as well as maliciously crafted
/// collisions.
pub const HANDLER_ID_BYTES: usize = 32;

/// An ID for an event handler that consists of random bytes. Uniqueness
/// depends on sufficient randomness in generation.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct HandlerId {
    bytes: [u8; HANDLER_ID_BYTES]
}

impl HandlerId {
    pub(crate) fn new() -> HandlerId {
        let mut rng = rand::thread_rng();
        HandlerId {
            bytes: rng.gen()
        }
    }
}
