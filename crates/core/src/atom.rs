use crate::lit::Lit;

pub trait Atom<Domains> {
    fn as_lit(&self, domains: &Domains) -> Lit;

    fn boxed_clone(&self) -> Box<dyn Atom<Domains>>;
    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<Domains> Atom<Domains> for Lit {
    fn as_lit(&self, _: &Domains) -> Lit {
        *self
    }

    fn boxed_clone(&self) -> Box<dyn Atom<Domains>> {
        Box::new(*self)
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
