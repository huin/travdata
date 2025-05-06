use std::{any::TypeId, hash::Hash, marker::PhantomData};

/// Describes an error encountered while interacting with an arena.
#[derive(Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ArenaError {
    InvalidToken(atree::Token, TypeId),
}

impl std::error::Error for ArenaError {}

impl std::fmt::Display for ArenaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ArenaError::*;
        match self {
            InvalidToken(token, type_id) => {
                write!(f, "invalid {:?} token: {:?}", type_id, token)
            }
        }
    }
}

/// A type-marked wrapper around [atree::Token].
#[derive(Debug)]
pub struct TypedToken<O>(atree::Token, PhantomData<O>)
where
    O: ?Sized;

// Workarounds for `derive` not being smart about `PhantomData<T>` not mattering in `TypedToken`.
// Using #[derive(Copy)] etc. is conditional on `T` also being `Copy`, etc., even though
// PhantomData makes it not matter.
impl<T> Clone for TypedToken<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for TypedToken<T> {}
impl<T> Eq for TypedToken<T> {}
impl<T> Hash for TypedToken<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<T> PartialEq for TypedToken<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

/// A type-marked wrapper around [atree::Arena].
pub struct TypedArena<O, I>(atree::Arena<I>, PhantomData<O>)
where
    O: ?Sized;

impl<O, I> TypedArena<O, I>
where
    O: 'static,
{
    pub fn new() -> Self {
        Self(atree::Arena::new(), PhantomData)
    }

    pub fn new_inner(&mut self, inner: I) -> TypedToken<O> {
        TypedToken::<O>(self.0.new_node(inner), PhantomData)
    }

    pub fn get_mut_inner(&mut self, token: TypedToken<O>) -> Result<&mut I, ArenaError> {
        self.0
            .get_mut(token.0)
            .map(|node| &mut node.data)
            .ok_or(ArenaError::InvalidToken(token.0, TypeId::of::<O>()))
    }

    pub fn get_inner(&self, token: TypedToken<O>) -> Result<&I, ArenaError> {
        self.0
            .get(token.0)
            .map(|node| &node.data)
            .ok_or(ArenaError::InvalidToken(token.0, TypeId::of::<O>()))
    }
}
