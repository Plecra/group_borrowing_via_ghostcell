use core::marker::PhantomData;
/// An invariant lifetime--required in order to make sure that a GhostCell can
/// be owned by a single ghost token.
#[derive(Clone, Copy, Default)]
pub struct InvariantLifetime<'id>(PhantomData<*mut &'id ()>);
impl<'id> InvariantLifetime<'id> {
    #[inline]
    pub const fn new() -> InvariantLifetime<'id> {
        InvariantLifetime(PhantomData)
    }
}