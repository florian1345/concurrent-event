//! This crate implements an event which executes registered event handlers
//! concurrently. Additionally, each handler is assigned an ID with which it
//! can be referenced after creation. This allows for associated state for each
//! event handler.
//!
//! The system was setup to be resistant to adversarial code. In particular,
//! IDs are assigned randomly with sufficiently many bits to be unfeasable to
//! guess. As a consequence, efficiency may suffer. Particularly the IDs are
//! larger than they would need to be if assigned sequentially.
//!
//! # Important Note
//!
//! We provide no guarantees as the security properties were not rigorously
//! verified. Do not use in critical systems without further investigation!
//!
//! # Example
//!
//! This is a simple usage scenarion with a custom event handler.
//!
//! ```
//! use concurrent_event::Event;
//! use concurrent_event::handler::EventHandler; 
//!
//! struct Printer;
//!
//! impl EventHandler<&str> for Printer {
//!     fn on_event(&mut self, arg: &str) {
//!         print!("{}", arg);
//!     }
//! }
//!
//! let mut event = Event::<&str, Printer>::new();
//! event.emit("Hello, World!");
//! ```
//!
//! In the `handler` package, default implementation for stateless and stateful
//! event handlers can be found, which take a closure at construction.

use std::collections::HashMap;
use std::marker::PhantomData;

use crossbeam::thread;

use crate::id::HandlerId;
use crate::handler::EventHandler;

pub mod id;
pub mod handler;

/// An event manages multiple handlers which can be registered.
///
/// # Type Parameters
///
/// * `A`: The type of event arguments which are distributed to the handlers.
/// * `H`: The type of event handlers which can be registered with this event.
/// To allow for different types, use `Box<dyn EventHandler<...>>`.
pub struct Event<A: Copy + Send, H: EventHandler<A>> {
    arg_type: PhantomData<A>,
    handlers: HashMap<HandlerId, H>
}

impl<A: Copy + Send, H: EventHandler<A>> Event<A, H> {

    /// Creates a new event without handlers.
    pub fn new() -> Event<A, H> {
        Event {
            arg_type: PhantomData,
            handlers: HashMap::new()
        }
    }

    /// Emits an event, invoking all currently registered handlers in parallel.
    /// If all event handlers terminated without panicking, `true` is returned.
    /// If any event handler panics, `false` is returned.
    ///
    /// # Parameters
    ///
    /// * `arg`: The event argument to dispatch.
    pub fn emit(&mut self, arg: A) -> bool {
        thread::scope(|s| {
            for handler in self.handlers.values_mut() {
                s.spawn(move |_| handler.on_event(arg));
            }
        }).is_ok()
    }

    /// Adds an event handler to notify for future events. A handler ID is
    /// returned, which can be used to identify the handler later.
    ///
    /// # Parameters
    ///
    /// * `handler`: The event handler to register.
    pub fn add_handler(&mut self, handler: H) -> HandlerId {
        let id = HandlerId::new();
        self.handlers.insert(id, handler);
        id
    }

    /// Gets a reference to the event handler registered under the given ID
    /// wrapped in a `Some` option variant. If no such handler is registered,
    /// `None` is returned.
    ///
    /// # Parameters
    ///
    /// * `id`: The handler ID for which to get the associated event handler.
    pub fn get_handler(&self, id: HandlerId) -> Option<&H> {
        self.handlers.get(&id)
    }
}

impl<'a, A: Copy + Send> Event<A, Box<dyn EventHandler<A> + 'a>> {

    /// Adds an event handler wrapped into a box to this event. This is mainly
    /// syntactic sugar for `event.add_handler(Box::new(handler))`.
    ///
    /// # Parameters
    ///
    /// * `handler`: The event handler to wrap in a box and register with this
    /// event.
    pub fn add_handler_boxed(&'a mut self, handler: impl EventHandler<A> + 'a) -> HandlerId {
        self.add_handler(Box::new(handler))
    }
}

#[cfg(test)]
mod test;
