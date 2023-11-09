use super::{Value, S32 as ComponentName};

pub trait Lifecycle {
    type Entity;

    fn create_object(
        &self,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Self::Entity, String>;

    fn create_arrow(
        &self,
        source: &Self::Entity,
        target: &Self::Entity,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Self::Entity, String>;

    fn add_descriptor(
        &self,
        target: &Self::Entity,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Self::Entity, String>;
}
