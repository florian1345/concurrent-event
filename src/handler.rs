//! Contains the definition of the event handler trait as well as some standard
//! implementations for common use cases.

/// A trait for event handlers which can be registered with an event. For
/// comfort, an implementation for `Box<dyn EventHandler<A>>` is provided.
///
/// # Type Parameters
///
/// * `A`: The type of event arguments accepted by this handler.
pub trait EventHandler<A> : Send {
    fn on_event(&mut self, arg: A);
}

/// An event handler which manages a simple closure that receives no state
/// except the event argument. The closure is executed every time an event is
/// received.
///
/// # Type Parameters
///
/// * `A`: The type of event arguments accepted by this handler.
///
/// # Example
///
/// ```
/// use concurrent_event::Event;
/// use concurrent_event::handler::StatelessEventHandler;
///
/// let mut ev = Event::<&str, StatelessEventHandler<&str>>::new();
/// let handler = StatelessEventHandler::new(|arg: &str| println!("{}", arg));
/// ev.add_handler(handler);
/// ev.emit("Hello World!");
/// ```
pub struct StatelessEventHandler<'a, A> {
    func: Box<dyn Fn(A) + Send + 'a>
}

impl<'a, A> StatelessEventHandler<'a, A> {
    /// Creates a new stateless event handler from a closure.
    ///
    /// # Parameters
    ///
    /// * `f`: A closure which is executed every time an event is received. It
    /// consumes the event argument.
    pub fn new(f: impl Fn(A) + Send + 'a) -> StatelessEventHandler<'a, A> {
        StatelessEventHandler {
            func: Box::new(f)
        }
    }
}

impl<'a, A> EventHandler<A> for StatelessEventHandler<'a, A> {
    fn on_event(&mut self, arg: A) {
        (self.func)(arg)
    }
}

/// An event handler which manages a closure together with some state which can
/// track information over multiple events. The closure is executed with a
/// mutable reference of the state every time an event is received.
///
/// # Type Parameters
///
/// * `A`: The type of event arguments accepted by this handler.
/// * `S`: The type of the state maintained by this handler.
///
/// # Example
///
/// ```
/// use concurrent_event::Event;
/// use concurrent_event::handler::StatefulEventHandler;
///
/// let mut ev = Event::<i32, StatefulEventHandler<i32, i32>>::new();
/// let handler = StatefulEventHandler::new(|arg: i32, state: &mut i32| *state += arg, 0);
/// let id = ev.add_handler(handler);
/// ev.emit(2);
/// ev.emit(3);
/// let state = *ev.get_handler(id).unwrap().state();
/// 
/// assert_eq!(5, state);
/// ```
pub struct StatefulEventHandler<'a, A, S: Send> {
    func: Box<dyn Fn(A, &mut S) + Send + 'a>,
    state: S
}

impl<'a, A, S: Send> StatefulEventHandler<'a, A, S> {

    /// Creates a new stateful event handler from a closure and the initial
    /// state.
    ///
    /// # Parameters
    ///
    /// * `f`: A closure which is executed every time an event is received. It
    /// consumes the event argument and gets a mutable reference to the current
    /// state.
    /// * `initial_state`: The initial state given to the closure in the first
    /// received event.
    pub fn new<F>(f: F, initial_state: S) -> StatefulEventHandler<'a, A, S>
    where
        F : Fn(A, &mut S) + Send + 'a
    {
        StatefulEventHandler {
            func: Box::new(f),
            state: initial_state
        }
    }

    /// Gets the current state.
    pub fn state(&self) -> &S {
        &self.state
    }
}

impl<'a, A, S: Send> EventHandler<A> for StatefulEventHandler<'a, A, S> {
    fn on_event(&mut self, arg: A) {
        (self.func)(arg, &mut self.state)
    }
}

impl<'a, A> EventHandler<A> for Box<dyn EventHandler<A> + 'a> {
    fn on_event(&mut self, arg: A) {
        self.as_mut().on_event(arg)
    }
}
