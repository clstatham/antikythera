use serde::{Deserialize, Serialize};

/// A cell that protects its inner value from being directly mutated except
/// through a deliberate interface.
///
/// This container implements `Deref` to `T`, but not `DerefMut`, to prevent
/// accidental mutation, even when you have a mutable reference to the
/// `ProtectedCell`. To mutate the inner value, you must use the `get_mut`
/// method, which requires a mutable reference to the `ProtectedCell` itself.
/// This makes it explicit when mutation is intended.
///
/// # Example
///
/// ```rust
/// # use antikythera::utils::ProtectedCell;
/// let mut cell = ProtectedCell::new(5);
/// assert_eq!(*cell, 5); // Deref works
///
/// // *cell += 1; // This line would cause a compile-time error
///
/// *ProtectedCell::get_mut(&mut cell) += 1; // Explicit mutation
/// assert_eq!(*cell, 6);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
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
