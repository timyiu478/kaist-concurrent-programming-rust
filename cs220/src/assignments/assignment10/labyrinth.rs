//! Labyrinth
//!
//! Look at `labyrinth_grade.rs` below before you start.
//! HINT: <https://en.wikipedia.org/wiki/100_prisoners_problem>
//!
//! NOTE: You will have to implement a probabilistic algorithm, which means, the algorithm can fail
//! even if you have implemented the solution. We recommend running multiple times (at least 5
//! times) to check your solution works well.

use std::cell::RefCell;

/// Husband
#[derive(Debug)]
pub struct Husband {
    brain: RefCell<[usize; 100]>,
}

impl Husband {
    /// What might a husband, who is looking for his wife's ID my_wife, be thinking?
    pub fn seeking(my_wife: usize) -> Self {
        let mut brain = [0; 100];
        brain[0] = my_wife; // his wife
        brain[1] = my_wife; // next room to check
        Husband {
            brain: RefCell::new(brain),
        }
    }

    #[allow(missing_docs)]
    pub fn has_devised_a_strategy(&self) -> Strategy<'_> {
        Strategy { husband: self }
    }

    /// Based on the information about currently visited room number and someone's wife ID trapped
    /// inside, what the husband should do next?
    pub fn carefully_checks_whos_inside(&self, room: usize, wife: usize) {
        let mut brain = self.brain.borrow_mut();
        if wife != brain[0] {
            brain[1] = wife;
        }
    }
}

/// Strategy of husband
#[derive(Debug)]
pub struct Strategy<'a> {
    husband: &'a Husband,
}

impl Iterator for Strategy<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.husband.brain.borrow()[1])
    }
}
