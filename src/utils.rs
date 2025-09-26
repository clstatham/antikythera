use serde::{Deserialize, Serialize};

/// A cell that protects its inner value from being
/// directly mutated except through a deliberate interface.
///
/// This container implements `Deref` to `T`, but not `DerefMut`, to prevent accidental mutation.
/// The only way to mutate the inner value is through the
/// `get_mut` method, which requires a mutable reference to the cell itself,
/// and cannot be used like a normal method (usage is `ProtectedCell::get_mut(&mut cell)`).
/// This protects against accidental mutation through dereferencing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtectedCell<T> {
    value: T,
}

impl<T> ProtectedCell<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(cell: &mut Self) -> &mut T {
        &mut cell.value
    }
}

impl<T> std::ops::Deref for ProtectedCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
