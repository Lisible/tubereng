pub mod system;

pub trait EntityDefinition {}
impl<A, B> EntityDefinition for (A, B) {}
