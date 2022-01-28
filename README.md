## Koval

A very simple IOC framework for rust.
Runtime, no macros. You can bind types to types and containers to containers to abstract implementation choices.

### Usage

```rust
use koval::{Container, Injectable, InjectionError};

pub trait FooServiceTrait: Send + Sync {
    fn foo_function(&self) -> bool;
}

pub type IFooService = Arc<dyn FooServiceTrait>;

pub struct FooServiceImpl {}

impl FooServiceTrait for FooServiceImpl {
    fn foo_function(&self) -> bool {
        true
    }
}

impl Injectable<IFooService> for FooServiceImpl {
    fn resolve_injectable(_: &Container) -> Result<IFooService, InjectionError> {
        Ok(Arc::new(FooServiceImpl {}))
    }
}

fn main()  {
    let container = Container::new()
        .bind_singleton::<IFooService, FooServiceImpl>()
        .build()
        .expect("Container failed to build");
    
    let foo_instance = container.resolve::<IFooService>().unwrap();

    assert_eq!(foo_instance.foo_function(), true);
}
```