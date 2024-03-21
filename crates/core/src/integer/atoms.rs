use crate::{
    atom::Atom,
    domains::{DomainId, DomainStore},
    lit::Lit,
};

use super::{BoundedInt, Int};

pub struct AtLeast<Dom> {
    pub(crate) domain: DomainId<Dom>,
    pub(crate) bound: Int,
}

impl<Domains, Dom> Atom<Domains> for AtLeast<Dom>
where
    Domains: DomainStore<Dom>,
    Dom: BoundedInt + 'static,
{
    fn as_lit(&self, domains: &Domains) -> Lit {
        let domain = &domains[self.domain.clone()];

        domain.lower_bound_lit(self.bound)
    }

    fn boxed_clone(&self) -> Box<dyn Atom<Domains>> {
        Box::new(AtLeast {
            domain: self.domain.clone(),
            bound: self.bound,
        })
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{:?} >= {}]", self.domain, self.bound)
    }
}

pub struct AtMost<Dom> {
    pub(crate) domain: DomainId<Dom>,
    pub(crate) bound: Int,
}

impl<Domains, Dom> Atom<Domains> for AtMost<Dom>
where
    Domains: DomainStore<Dom>,
    Dom: BoundedInt + 'static,
{
    fn as_lit(&self, domains: &Domains) -> Lit {
        let domain = &domains[self.domain.clone()];

        domain.upper_bound_lit(self.bound)
    }

    fn boxed_clone(&self) -> Box<dyn Atom<Domains>> {
        Box::new(AtMost {
            domain: self.domain.clone(),
            bound: self.bound,
        })
    }

    fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{:?} <= {}]", self.domain, self.bound)
    }
}
