use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref};
use crate::Injectable;
use std::sync::Arc;
use crate::InjectionError;

pub type ResolutionFn = Arc<dyn Fn(&mut Container) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> + Send + Sync + 'static>;

#[derive(Eq, PartialEq, Clone)]
pub enum ResolutionType {
    Singleton,
    Transient,
}

#[derive(Clone)]
pub struct Resolution {
    pub resolution_type: ResolutionType,
    pub stored_instance: Option<Arc<dyn Any + Send + Sync>>,
    pub resolution_fn: ResolutionFn,
}

#[derive(Clone)]
pub struct Container {
    bindings: HashMap<TypeId, Resolution>,
}

fn wrap_injectable<T, Fi>(inj_fun: &'static Fi) -> ResolutionFn
where Fi: 'static + Send + Sync + Fn(&mut Container) -> Result<T, InjectionError>, Result<T, InjectionError>: 'static, T: Send + Sync
{
    Arc::new(|cont: &mut Container| -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        let resolved = inj_fun(cont)?;

        Ok(Arc::new(resolved))
    })
}

impl Container {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind_singleton<InjTraitType, ActType>(mut self) -> Self
    where InjTraitType: 'static + Sized + Send + Sync,
        ActType: Injectable<InjTraitType> + 'static + Send + Sync,
    {
        let id = TypeId::of::<InjTraitType>();
        self.bindings.insert(id, Resolution {
            resolution_type: ResolutionType::Singleton,
            stored_instance: None,
            resolution_fn: wrap_injectable( &ActType::resolve_injectable),
        });

        self
    }

    pub fn bind_transient<InjTraitType, ActType>(mut self) -> Self
        where InjTraitType: 'static + Sized + Send + Sync,
              ActType: Injectable<InjTraitType> + 'static + Send + Sync,
    {
        let id = TypeId::of::<InjTraitType>();
        self.bindings.insert(id, Resolution {
            resolution_type: ResolutionType::Transient,
            stored_instance: None,
            resolution_fn: wrap_injectable( &ActType::resolve_injectable),
        });

        self
    }

    fn downcast_resolved<T: 'static + Clone>(resolved: Arc<dyn Any + Send + Sync>) -> Result<T, InjectionError> {
        resolved
            .deref()
            .downcast_ref::<T>()
            .map(|cc| cc.clone())
            .ok_or(InjectionError::TypeNotBound)
    }

    /// In case of non stored singeleton, this resolve fn will store the resolution result
    pub fn resolve_mut<InjTraitType>(&mut self) -> Result<InjTraitType, InjectionError>
    where InjTraitType: 'static + Clone
    {
        let id = TypeId::of::<InjTraitType>();
        let mut binding = self.bindings.get_mut(&id)
            .ok_or(InjectionError::TypeNotBound)?.clone();


        if binding.resolution_type == ResolutionType::Singleton {
            if let Some(inst) = binding.stored_instance.as_ref() {
                Self::downcast_resolved(inst.clone())
            } else {
                let resolved = (binding.resolution_fn)(self)?;
                binding.stored_instance = Some(resolved.clone());
                self.bindings.insert(id, binding);
                Self::downcast_resolved(resolved)
            }
        }
        else if binding.resolution_type == ResolutionType::Transient {
            let resolved = (binding.resolution_fn)(self)?;
            Self::downcast_resolved(resolved)
        }
        else {
            unreachable!();
        }
    }

    /// In case of non stored singeleton, this resolve fn will NOT store the resolution result
    pub fn resolve<InjTraitType>(&self) -> Result<InjTraitType, InjectionError>
        where InjTraitType: 'static + Clone
    {
        let id = TypeId::of::<InjTraitType>();
        let binding = self.bindings.get(&id)
            .ok_or(InjectionError::TypeNotBound)?;


        if binding.resolution_type == ResolutionType::Singleton {
            if let Some(inst) = binding.stored_instance.as_ref() {
                Self::downcast_resolved(inst.clone())
            } else {
                // Bit costy but hey
                let resolved = (binding.resolution_fn)(&mut self.clone())?;
                Self::downcast_resolved(resolved)
            }
        }
        else if binding.resolution_type == ResolutionType::Transient {
            // Bit costy but hey
            let resolved = (binding.resolution_fn)(&mut self.clone())?;
            Self::downcast_resolved(resolved)
        }
        else {
            unreachable!();
        }
    }

    pub fn bind_container_into(mut self, container: Self) -> Self {
        for (key, val) in container.bindings.into_iter() {
            self.bindings.insert(key, val);
        }

        self
    }

    /// tires to store all singeletons so later the container can be used with a non mut resolve fn
    pub fn build(mut self) -> Result<Self, InjectionError> {
        let old_instance = self.clone();
        for (id, val) in old_instance.bindings.iter() {
            let current_has_stored_instance = self.bindings.get(id)
                .map(|val| val.stored_instance.is_some())
                .unwrap_or(false);
            if val.resolution_type == ResolutionType::Singleton && !current_has_stored_instance {
                (val.resolution_fn)(&mut self)?;
            }
        }

        Ok(self)
    }
}