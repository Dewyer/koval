#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{Container, Injectable, InjectionError};

    trait ServiceTrait {
        fn do_something(&self) -> i32;
    }

    struct ServiceImpl {
    }

    impl ServiceTrait for ServiceImpl {
        fn do_something(&self) -> i32 {
            10
        }
    }

    impl Injectable<Arc<dyn ServiceTrait>> for ServiceImpl {
        fn resolve_injectable(_: &Container) -> Result<Arc<dyn ServiceTrait>, InjectionError> {
            Ok(Arc::new(Self {}))
        }
    }


    #[test]
    fn it_works_unbuilt() {
        let container = Container::new()
            .bind_singleton::<Arc<dyn ServiceTrait>, ServiceImpl>();

        let srv = container.resolve::<Arc<dyn ServiceTrait>>()
            .expect("rip lol");

        assert_eq!(srv.do_something(),10);
    }

    #[test]
    fn it_works_built() {
        let container = Container::new()
            .bind_singleton::<Arc<dyn ServiceTrait>, ServiceImpl>()
            .build().unwrap();

        let srv = container.resolve::<Arc<dyn ServiceTrait>>()
            .expect("rip lol");

        assert_eq!(srv.do_something(),10);
    }

    #[test]
    fn it_works_built_w_child() {
        let child_container = Container::new()
            .bind_singleton::<Arc<dyn ServiceTrait>, ServiceImpl>();

        let container = Container::new()
            .bind_container_into(child_container)
            .build().unwrap();

        let srv = container.resolve::<Arc<dyn ServiceTrait>>()
            .expect("rip lol");

        assert_eq!(srv.do_something(),10);
    }

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

    fn main_er()  {
        let container = Container::new()
            .bind_singleton::<IFooService, FooServiceImpl>()
            .build()
            .expect("Container failed to build");

        let foo_instance = container.resolve::<IFooService>().unwrap();

        assert_eq!(foo_instance.foo_function(), true);
    }

    #[test]
    fn readme_test() {
        main_er()
    }
}
