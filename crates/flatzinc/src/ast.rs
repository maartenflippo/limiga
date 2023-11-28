use std::{ops::Deref, rc::Rc};

/// The integer type used.
///
/// The FlatZinc specification does not specify what size integers we need to support, nor
/// does it give use the option to indicate the size of our integers. This means we will
/// just crash whenever a model is run that does not fit into this integer type. Practically
/// speaking, if that happens, the problem likely cannot be solved anyways so for now this
/// is not really a concern.
pub type Int = i64;

#[derive(Debug, PartialEq, Eq)]
pub enum ModelItem {
    Parameter(Parameter),
}

/// A parameter declaration.
#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    pub identifier: Identifier,
    pub value: Value,
}

/// A FlatZinc identifier. Supports cheap cloning as it is a reference-counted string slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identifier(Rc<str>);

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> From<&'a str> for Identifier {
    fn from(value: &'a str) -> Self {
        Identifier(value.into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(Int),
    Bool(bool),
}
