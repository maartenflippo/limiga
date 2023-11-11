use std::{fmt::Debug, ops::Not};

use thiserror::Error;

use crate::storage::Indexer;

/// A boolean variable.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Var(u32);

impl Var {
    /// Create a new variable from the given code.
    ///
    /// # Safety
    /// The code *must* be larger than [`MAX_VAR_CODE`].
    pub unsafe fn new_unchecked(code: u32) -> Var {
        Var(code)
    }

    #[inline]
    pub fn code(self) -> u32 {
        self.0
    }
}

impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x{}", self.0)
    }
}

impl Indexer for Var {
    fn index(&self) -> usize {
        self.0 as usize
    }
}

pub const MAX_VAR_CODE: u32 = (!0) >> 1;

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("The value {0} is larger than the maximum variable code {MAX_VAR_CODE}.")]
pub struct VarCodeTooBig(u32);

impl TryFrom<u32> for Var {
    type Error = VarCodeTooBig;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > MAX_VAR_CODE {
            Err(VarCodeTooBig(value))
        } else {
            Ok(Var(value))
        }
    }
}

/// A literal is a signed boolean variable, either positive or negative.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Lit(u32);

impl Lit {
    /// Create a positive literal from the given variable.
    #[inline]
    pub fn positive(var: Var) -> Lit {
        let code = (var.0 << 1) | 1;
        Lit(code)
    }

    /// Create a negative literal from the given variable.
    #[inline]
    pub fn negative(var: Var) -> Lit {
        let code = var.0 << 1;
        Lit(code)
    }

    #[inline]
    pub fn is_positive(self) -> bool {
        self.0 & 1 == 1
    }

    #[inline]
    pub fn is_negative(self) -> bool {
        self.0 & 1 == 0
    }

    #[inline]
    pub fn var(self) -> Var {
        Var(self.0 >> 1)
    }
}

impl Debug for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_positive() {
            write!(f, "{:?}", self.var())
        } else {
            write!(f, "-{:?}", self.var())
        }
    }
}

impl Indexer for Lit {
    fn index(&self) -> usize {
        self.0 as usize
    }
}

impl Not for Lit {
    type Output = Lit;

    fn not(self) -> Self::Output {
        Lit(self.0 ^ 1)
    }
}

#[macro_export]
macro_rules! lit {
    (-$code:literal) => {
        $crate::lit::Lit::negative($crate::lit::Var::new_unchecked($code))
    };

    ($code:literal) => {
        $crate::lit::Lit::positive($crate::lit::Var::new_unchecked($code))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn var_construction() {
        let var = Var::try_from(0);
        assert_eq!(Ok(Var(0)), var);

        let var = Var::try_from(MAX_VAR_CODE + 1);
        assert_eq!(Err(VarCodeTooBig(MAX_VAR_CODE + 1)), var);
    }

    #[test]
    fn literal_construction() {
        let positive = Lit::positive(Var(0));
        let negative = Lit::negative(Var(1));

        assert!(positive.is_positive());
        assert!(negative.is_negative());

        assert_eq!(Var(0), positive.var());
        assert_eq!(Var(1), negative.var());
    }

    #[test]
    fn literal_macro() {
        let positive = unsafe { lit!(1) };
        let negative = unsafe { lit!(-1) };

        assert_eq!(positive.var(), negative.var());
        assert!(positive.is_positive());
        assert!(negative.is_negative());
    }
}
