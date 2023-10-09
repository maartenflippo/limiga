use std::ops::Index;

use crate::lit::Lit;

#[derive(Default)]
pub struct Trail {
    trail: Vec<Lit>,
    trail_delim: Vec<usize>,
}

impl Trail {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.trail.len()
    }

    pub fn enqueue(&mut self, lit: Lit) {
        self.trail.push(lit);
    }

    pub fn push(&mut self) {
        self.trail_delim.push(self.trail.len());
    }

    pub fn backtrack_to(&mut self, decision_level: usize) -> impl Iterator<Item = Lit> + '_ {
        let target_len = self.trail_delim[decision_level];
        self.trail_delim.truncate(decision_level);

        let current = self.trail.len();

        BacktrackingIterator {
            trail: self,
            target_len,
            current,
        }
    }

    pub fn pop(&mut self) -> Option<Lit> {
        self.trail.pop()
    }
}

impl Index<usize> for Trail {
    type Output = Lit;

    fn index(&self, index: usize) -> &Self::Output {
        &self.trail[index]
    }
}

struct BacktrackingIterator<'a> {
    trail: &'a mut Trail,
    current: usize,
    target_len: usize,
}

impl<'a> Iterator for BacktrackingIterator<'a> {
    type Item = Lit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > 0 && self.current > self.target_len {
            self.current -= 1;
            Some(self.trail[self.current])
        } else {
            None
        }
    }
}

impl<'a> Drop for BacktrackingIterator<'a> {
    fn drop(&mut self) {
        self.trail.trail.truncate(self.target_len);
    }
}

#[cfg(test)]
mod tests {
    use crate::lit;

    use super::*;

    #[test]
    fn len_of_trail_is_number_of_enqueued_lits() {
        let mut trail = Trail::default();
        assert_eq!(0, trail.len());

        trail.enqueue(unsafe { lit!(1) });
        trail.enqueue(unsafe { lit!(2) });

        assert_eq!(2, trail.len());
    }

    #[test]
    fn backtracking_returns_iterator_with_removed_literals() {
        let mut trail = Trail::default();

        trail.enqueue(unsafe { lit!(1) });
        trail.enqueue(unsafe { lit!(2) });
        trail.enqueue(unsafe { lit!(3) });
        trail.push();
        trail.enqueue(unsafe { lit!(4) });
        trail.enqueue(unsafe { lit!(5) });
        trail.enqueue(unsafe { lit!(6) });
        trail.push();
        trail.enqueue(unsafe { lit!(7) });
        trail.enqueue(unsafe { lit!(8) });
        trail.enqueue(unsafe { lit!(9) });

        let removed_lits = trail.backtrack_to(0).collect::<Vec<_>>();
        assert_eq!(
            unsafe { vec![lit!(9), lit!(8), lit!(7), lit!(6), lit!(5), lit!(4)] },
            removed_lits
        );
    }
}
