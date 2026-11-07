use std::any::Any;

pub(super) struct Value(Box<dyn Any + Send>);

impl Value {
    pub(super) fn new(value: impl Any + Send) -> Self {
        Self(Box::new(value))
    }

    pub(super) fn downcast_ref<T: Any>(&self) -> &T {
        self.0
            .downcast_ref()
            .expect("value type should match its request")
    }
}
