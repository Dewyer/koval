use crate::Container;
use crate::InjectionError;

pub trait Injectable<ForTrait> {
    fn resolve_injectable(container: &mut Container) -> Result<ForTrait, InjectionError>;
}

pub trait FromContainer: Sized {
    fn from_container(container: &mut Container) -> Result<Self, InjectionError>;
}