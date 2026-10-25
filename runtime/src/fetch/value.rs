use std::any::Any;

pub(super) struct Value(Box<dyn Any>);

impl Value {
    pub(super) fn new(value: impl Any) -> Self {
        Self(Box::new(value))
    }

    pub(super) fn into_inner<T: Any>(self) -> T {
        *self
            .0
            .downcast::<T>()
            .expect("value type should match its computation")
    }

    pub(super) fn downcast_ref<T: Any>(&self) -> &T {
        self.0
            .downcast_ref()
            .expect("value type should match its computation")
    }
}
