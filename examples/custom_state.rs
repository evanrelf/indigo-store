#![allow(dead_code)]

use indigo_store::{Field, Store, TypeMap, type_map};
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};

struct State {
    name: Name,
    extras: TypeMap,
}

#[derive(Debug, PartialEq)]
struct Name(String);

impl Field<Name> for State {
    type Error = Infallible;

    fn field(&self) -> Result<&Name, Self::Error> {
        Ok(&self.name)
    }

    fn field_mut(&mut self) -> Result<&mut Name, Self::Error> {
        Ok(&mut self.name)
    }
}

enum NameAction {
    Renamed(String),
    Cleared,
}

fn name_reducer(state: &mut Name, action: &NameAction) {
    match action {
        NameAction::Renamed(name) => state.0.clone_from(name),
        NameAction::Cleared => state.0.clear(),
    }
}

#[derive(Debug, PartialEq)]
struct Count(isize);

impl Field<Count> for State {
    type Error = ();

    fn field(&self) -> Result<&Count, Self::Error> {
        self.extras.field()
    }

    fn field_mut(&mut self) -> Result<&mut Count, Self::Error> {
        self.extras.field_mut()
    }
}

enum CountAction {
    Incremented,
    Decremented,
}

fn count_reducer(state: &mut Count, action: &CountAction) {
    match action {
        CountAction::Incremented => state.0 += 1,
        CountAction::Decremented => state.0 -= 1,
    }
}

fn main() {
    let mut store = Store::new(State {
        name: Name("Alice".to_string()),
        extras: type_map![Count(0)],
    });

    // Add a reducer function to modify `Count` in response to `CountAction`
    store.add_reducer(count_reducer);

    // Add a listener function to perform effects in response to changes to `Count`
    let count_changes = Arc::new(Mutex::new(0u8));
    let listener_count_changes = Arc::clone(&count_changes);
    store.add_listener(move |_: &Count| {
        *listener_count_changes.lock().unwrap() += 1;
    });

    // Add a reducer function to modify `Name` in response to `NameAction`
    store.add_reducer(name_reducer);

    // Dispatch `CountAction`s to modify `Count`
    store.dispatch(CountAction::Incremented);
    store.dispatch(CountAction::Incremented);
    store.dispatch(CountAction::Decremented);

    // Dispatch `NameAction`s to modify `Name`
    store.dispatch(NameAction::Renamed("Bob".to_string()));

    assert_eq!(store.field().unwrap() as &Count, &Count(1));
    assert_eq!(*count_changes.lock().unwrap(), 3);
    assert_eq!(store.name, Name("Bob".to_string()));
}
