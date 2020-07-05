use std::thread;
use std::time::{Duration, Instant};

use super::*;

use crate::handler::StatelessEventHandler;
use crate::handler::StatefulEventHandler;

#[test]
fn empty_emit() {
    // We only test nothing panics
    let mut ev = Event::<i32, StatelessEventHandler<i32>>::new();
    assert!(ev.emit(5));
}

fn setup_single_handler_event<'a>() ->
        (Event<i32, StatefulEventHandler<'a, i32, i32>>, HandlerId) {
    let mut ev = Event::<i32, StatefulEventHandler<i32, i32>>::new();
    let handler = StatefulEventHandler::new(|arg, state| *state += arg, 0);
    let id = ev.add_handler(handler);
    (ev, id)
}

#[test]
fn single_handler_single_event() {
    let (mut ev, id) = setup_single_handler_event();
    assert!(ev.emit(5));

    let new_state = *ev.get_handler(id).unwrap().state();
    assert_eq!(5, new_state);
}

#[test]
fn single_handler_multiple_events() {
    let (mut ev, id) = setup_single_handler_event();
    assert!(ev.emit(5));
    assert!(ev.emit(3));

    let new_state = *ev.get_handler(id).unwrap().state();
    assert_eq!(8, new_state);
}

#[test]
fn multiple_handlers_multiple_events() {
    let mut ev = Event::<i32, StatefulEventHandler<i32, i32>>::new();
    let h1 = StatefulEventHandler::new(|arg, state| *state += arg, 0);
    let h2 = StatefulEventHandler::new(|arg, state| *state *= arg, 1);
    let id1 = ev.add_handler(h1);
    let id2 = ev.add_handler(h2);

    assert!(ev.emit(3));
    assert!(ev.emit(5));

    let new_state_1 = *ev.get_handler(id1).unwrap().state();
    let new_state_2 = *ev.get_handler(id2).unwrap().state();
    assert_eq!(8, new_state_1);
    assert_eq!(15, new_state_2);
}

#[test]
fn boxed_handler() {
    // We only test nothing panics and emit(...) terminates.
    let mut ev = Event::<i32, Box<dyn EventHandler<i32>>>::new();
    let handler = Box::new(StatelessEventHandler::new(|_: i32| { }));
    ev.add_handler(handler);
    ev.emit(7);
}

#[test]
fn parallel_execution() {
    let mut ev = Event::<(), StatelessEventHandler<()>>::new();
    let duration = Duration::from_millis(16);
    let handler_count = 32;

    for _ in 0..handler_count {
        ev.add_handler(StatelessEventHandler::new(move |_| thread::sleep(duration)));
    }

    let before = Instant::now();
    assert!(ev.emit(()));
    let elapsed = before.elapsed();

    assert!(elapsed < duration * (handler_count / 2));
}

#[test]
fn awaits() {
    let mut ev = Event::<(), StatefulEventHandler<(), bool>>::new();
    let handler = StatefulEventHandler::new(|_: (), s| *s = true, false);
    let id = ev.add_handler(handler);

    assert!(ev.emit(()));

    let state = *ev.get_handler(id).unwrap().state();
    assert!(state);
}

struct PanicState {
    panicked: bool,
    calmed: bool
}

impl PanicState {
    fn new() -> PanicState {
        PanicState {
            panicked: false,
            calmed: false
        }
    }
}

#[test]
fn panicking() {
    let mut ev = Event::<(), StatefulEventHandler<(), PanicState>>::new();
    let handler = StatefulEventHandler::new(|_: (), s| {
        if s.panicked {
            s.calmed = true;
        }
        else {
            s.panicked = true;
            panic!("(╯°□°）╯︵ ┻━┻");
        }
    }, PanicState::new());
    let id = ev.add_handler(handler);

    assert!(!ev.emit(()));
    assert!(ev.emit(()));

    let new_state = ev.get_handler(id).unwrap().state();
    assert!(new_state.calmed);
}
