use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    fmt::Debug,
    rc::{Rc, Weak},
};

use crate::object::Object;

const MAX_OBJECTS: usize = 10_000;
const MIN_INSTRUCTIONS: usize = 500;

#[derive(Debug, Clone)]
pub struct Ref<T> {
    pub value: Weak<RefCell<T>>,
    pub id: usize,
}

struct Owner {
    value: Rc<dyn Any>,
    /// Is marked for dealocation.
    marked: bool,
}

impl Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Owner")
            .field("value", &self.value)
            .field("marked", &self.marked)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct GarbageCollector {
    owners: HashMap<usize, Owner>,
    nr_instructions: usize,
}

impl GarbageCollector {
    pub fn should_free(&mut self) -> bool {
        self.nr_instructions += 1;

        if self.owners.len() > MAX_OBJECTS && self.nr_instructions > MIN_INSTRUCTIONS {
            self.nr_instructions = 0;
            true
        } else {
            false
        }
    }

    pub fn allocate<T: 'static>(&mut self, val: T) -> Ref<T> {
        let rc = Rc::new(RefCell::new(val));
        let id = rc.as_ptr() as usize;

        let rc_ref = Ref {
            value: Rc::downgrade(&rc),
            id,
        };

        self.owners.insert(
            id,
            Owner {
                value: rc,
                marked: false,
            },
        );

        rc_ref
    }

    pub fn free(&mut self, used_stack: &[Object], globals: &[Object]) {
        self.mark_all(true);

        for obj in used_stack {
            self.traverse(obj);
        }

        for obj in globals {
            self.traverse(obj)
        }

        self.owners.retain(|_, owner| !owner.marked);
    }

    fn traverse(&mut self, obj: &Object) {
        match obj {
            Object::Array(arr) => {
                self.set_mark(arr.0.id, false);
                for val in arr
                    .0
                    .value
                    .upgrade()
                    .expect("Accessing freed value")
                    .borrow()
                    .iter()
                {
                    self.traverse(val);
                }
            }
            Object::Dictionary(dict) => {
                self.set_mark(dict.0.id, false);
                for val in dict
                    .0
                    .value
                    .upgrade()
                    .expect("Accessing freed value")
                    .borrow()
                    .values()
                {
                    self.traverse(val);
                }
            }
            Object::Closure(closure) => {
                for val in closure.free_variables.iter() {
                    self.traverse(val);
                }
            }
            _ => (),
        }
    }

    fn mark_all(&mut self, value: bool) {
        for (_, val) in self.owners.iter_mut() {
            val.marked = value;
        }
    }

    fn set_mark(&mut self, id: usize, value: bool) {
        if let Some(owner) = self.owners.get_mut(&id) {
            owner.marked = value;
        }
    }
}
