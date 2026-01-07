/*
All files in this development are distributed under the terms of the BSD
license, included below.

------------------------------------------------------------------------------

                            BSD LICENCE

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the <organization> nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL <COPYRIGHT HOLDER> BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
/*
COPYRIGHT
The LICENSE above is from https://gitlab.mpi-sws.org/FP/ghostcell/-/blob/master/LICENSE
So I'm gonna say the license holder is "MPI-SWS" in lieu of it being specified.
 */


//! Unsafe: tread carefully!
//!
//! # GhostCell: A cell with ghost ownership.
//!
//! Often, interior mutability in Rust comes with unsatisfying tradeoffs: thread
//! safety, generality of types, runtime overhead (both time and space) or even
//! runtime failure.
//! To avoid some of these issues, one can improve interior mutability by taking
//! advantage of Rust's ownership, but the trick is unfortunately
//! limited to `Cell`s where one has direct ownership.
//!
//! Here, we extend this trick to `GhostCell` where one has *logical* (ghost)
//! ownership, with the following techniques:
//!
//! - Invariant lifetimes (to generate unforgeable, compile-time unique lifetime
//!   tokens).
//! - Higher rank lifetimes (to ensure code is parametric with respect to the
//!   chosen token).
//! - Ghost ownership (to allow ownership to be held by the token's owner, rather
//!   than by the cells themselves).
//!
//! The API works as follows:
//! 1. create a `GhostToken`, within an appropriate scope (a "context").
//! 2. create cells that reference the ghost token. They must reference exactly
//!    one ghost token, due to lifetime invariance, and no two ghost tokens can
//!    be the same, due to lifetime parametricity.
//! 3. to access values at a given lifetime and mutability, borrow the token
//!    with that lifetime and mutability. As a result, Rust's guarantees about
//!    ownership and lifetimes flow from the token to its owned cells.
//!
//! [The methods provided by this type have been formally verified in Coq.](http://plv.mpi-sws.org/rustbelt/ghostcell/)
// pub mod dlist_arc;
// pub mod dlist_arena;
// pub mod dfs_arena;
// pub mod list_arena;
// pub mod dfs_arena_list;
use core::{cell::UnsafeCell, marker::PhantomData};
pub type InvariantLifetime<'brand> = generativity::Guard<'brand>;
/// A ghost token.
///
/// Once created, a `GhostToken` can neither be cloned nor copied.
///
/// Note that `GhostToken` does not need to know which types it is protecting. The
/// reason is that in order to do anything with a `GhostCell<T>`, both the token
/// *and* a reference to the `GhostCell` are required. Since a `GhostCell` inherits
/// trait markers (and other information) from the type `T` of data it is
/// protecting, `GhostToken` does not need to; it only needs to carry around
/// ownership of the entire set of `GhostCell`s.
/// For example, one could create a `GhostCell<'id, Rc<T>>` and then send its
/// owning `GhostToken<'id>` to another thread, but since one cannot actually send
/// the `GhostCell` to another thread, it is not possible to create a race
/// condition in this way.
///
/// Note also that `'id` is totally disjoint from `T` itself--it is only used as
/// a unique compile-time identifier for a set of (`GhostCell`s of) `T`s, and
/// otherwise has no relationship to `T`.

// pub struct GhostToken<'id> {
//     _marker: InvariantLifetime<'id>,
// }
// GhostToken are always instantiated with a Guard<'id>, but we rely on the generic
// parameter to allow traits to carry "associated lifetimes"
pub type GhostToken<r> = PhantomData<r>;
/// A ghost cell.
///
/// A `GhostCell` acts exactly like a `T`, except that its contents are
/// accessible only through its owning `GhostToken`.
#[derive(Default)]
#[repr(transparent)]
pub struct GhostCell<C, T: ?Sized> {
    _marker: PhantomData<C>,
    value: UnsafeCell<T>, // invariant in `T`
}
/// `GhostCell<'id, T>` implements `Send` iff `T` does. This is safe because in
/// order to access the `T` mutably within a `GhostCell<T>`, you need both a
/// mutable reference to its owning `GhostToken` and an immutable reference to
/// `GhostCell<T>`, and both references must have the same lifetime.
unsafe impl<C, T> Send for GhostCell<C, T> where T: Send {}
/// `GhostCell<'id, T>` implements `Sync` iff `T` is `Send + Sync`. This is safe
/// because in order to access the `T` immutably within a `GhostCell<T>`, you
/// need both an immutable reference to its owning `GhostToken` and an immutable
/// reference to `GhostCell<T>`, and both references must have the same lifetime.
unsafe impl<C, T> Sync for GhostCell<C, T> where T: Send + Sync {}

impl<'id, T> GhostCell<InvariantLifetime<'id>, T> {
    /// Creates a new cell that belongs to the token at lifetime `'id`. This
    /// consumes the value of type `T`. From this point on, the only way to access
    /// the inner value is by using a `GhostToken` with the same `'id`. Since
    /// `'id` is always chosen parametrically, and `'id` is invariant for both
    /// the `GhostCell` and the `GhostToken`, if one chooses `'id` to correspond

    /// to an existing `GhostToken<'id>`, that is the only `GhostToken<'id>` to
    /// which the `GhostCell` belongs. Therefore, there is no way to access the
    /// value through more than one token at a time.
    ///
    /// As with `GhostToken` itself, note that `'id` has no relationship to
    /// `T`---it is only used as a unique, static marker.
    ///
    /// A subtle point to make is around `Drop`. If `T`'s `Drop` implementation
    /// is run, and `T` has a reference to a `GhostToken<'id>`, it seems that
    /// since the invariant that `GhostToken<'id>` must be accessed mutably to
    /// get a mutable reference to the `T` inside a `GhostCell<'id, T>` is being
    /// bypassed, there could be a soundness bug here. Fortunately, thanks to
    /// `dropck`, such pathological cases appear to be ruled out. For example,
    /// this code will not compile:
    ///
    /// ```compile_fail
    /// use ghost_cell::{GhostToken, GhostCell};
    ///
    /// struct Foo<'a, 'id>(&'a GhostToken<'id>,
    ///                         GhostCell<'id, GhostCell<Option<&'a Foo<'a, 'id>>>>)
    ///                              where 'id: 'a;
    ///
    /// impl<'a, 'id> Drop for Foo<'a, 'id> {
    ///     fn drop(&mut self) {
    ///         match self.0.get(&self.1).get() {
    ///             Some(ref foo) => {
    ///                 println!("Oops, have aliasing.");
    ///             },
    ///             None => {
    ///                 println!("Okay")
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// GhostToken::new(|token| {
    ///     let foo = Foo(&token, GhostCell::new(GhostCell::new(None)));
    ///     foo.1.borrow(&token).set(Some(&foo));
    /// });
    /// ```
    ///
    /// It will compile if the manual `Drop` implementation is removed, but only
    /// pathological `Drop` implementations are an issue here.  I believe there
    /// are two factors at work: one, in order to have a reference to a value,
    /// the token must outlive the reference. Two, types must *strictly* outlive
    /// the lifetimes of things they reference if they have a nontrivial `Drop`
    /// implementation.  As a result, if there is any reference to a `GhostCell`
    /// containing the type being dropped from within the type being dropped,
    /// and it has a nontrivial `Drop` implementation, it will not be possible to
    /// complete the cycle.  To illustrate more clearly, this fails, too:
    ///
    /// ```compile_fail
    /// fn foo() {
    ///     struct Foo<'a>(GhostCell<Option<&'a Foo<'a>>>);
    ///
    ///     impl<'a> Drop for Foo<'a> {
    ///         fn drop(&mut self) {}
    ///     }
    ///
    ///     let foo = Foo(GhostCell::new(None));
    ///     foo.0.set(Some(&foo));
    /// }
    /// ```
    ///
    /// So any conceivable way to peek at a self-reference within a `Drop`
    /// implementation is probably covered.
    #[inline]
    pub const fn new(value: T) -> Self {
        GhostCell {
            _marker: PhantomData,
            value: UnsafeCell::new(value),
        }
    }
    /// Unwraps the value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
    /// Get an immutable reference to the item that lives for as long as the
    /// owning token is immutably borrowed (the lifetime `'a`).
    #[inline]
    pub fn borrow<'a>(&'a self, _token: &'a InvariantLifetime<'id>) -> &'a T {
        unsafe {
            // We know the token and lifetime are both borrowed at 'a, and the
            // token is borrowed immutably; therefore, nobody has a mutable
            // reference to this token. Therefore, any items in the set that are
            // currently aliased would have been legal to alias at &'a T as well,
            // so we can take out an immutable reference to any of them, as long
            // as we make sure that nobody else can take a mutable reference to
            // any item in the set until we're done.
            &*self.value.get()
        }
    }
    /// Get a mutable reference to the item that lives for as long as the owning
    /// token is mutably borrowed.
    #[inline]
    pub fn borrow_mut<'a>(&'a self, _token: &'a mut InvariantLifetime<'id>) -> &'a mut T {
        unsafe {
            // We know the token and lifetime are both borrowed at `'a`, and the
            // token is borrowed mutably; therefore, nobody else has a mutable 
            // reference to this token.  As a result, all items in the set are
            // currently unaliased, so we can take out a mutable reference to
            // any one of them, as long as we make sure that nobody else can
            // take a mutable reference to any other item in the set until
            // the current borrow is done.
            &mut *self.value.get()
        }
    }
}
impl<'id, T> From<T> for GhostCell<InvariantLifetime<'id>, T> {
    #[inline]
    fn from(t: T) -> Self {
        GhostCell::new(t)
    }
}
impl<'id, T: ?Sized> GhostCell<InvariantLifetime<'id>, T> {
    /// Returns a raw pointer to the underlying data in this cell.
    pub const fn as_ptr(&self) -> *mut T {
        self.value.get()
    }
    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows `GhostCell` mutably (at compile-time) which guarantees
    /// that we possess the only reference.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }
    /// Returns a `&mut GhostCell<'id, T>` from a `&mut T`
    #[inline]
    pub fn from_mut(t: &mut T) -> &mut Self {
        unsafe { &mut *(t as *mut T as *mut Self) }

    }
}
impl<'id, T> GhostCell<InvariantLifetime<'id>, [T]> {
    /// Returns a `&[GhostCell<'id, T>]` from a `&GhostCell<'id, [T]>`
    #[inline]
    pub fn as_slice_of_cells(&self) -> &[GhostCell<InvariantLifetime<'id>, T>] {
        unsafe { &*(self as *const GhostCell<InvariantLifetime<'id>, [T]> as *const [GhostCell<InvariantLifetime<'id>, T>]) }
    }
}
impl<'id, T: Clone> GhostCell<InvariantLifetime<'id>, T> {
    /// Convenience method to clone the `GhostCell` when `T` is `Clone`, as long
    /// as the token is available.
    #[inline]
    pub fn clone(&self, token: &InvariantLifetime<'id>) -> Self {
        GhostCell::new(self.borrow(token).clone())
    }
}
impl<'id, T: ?Sized> AsMut<T> for GhostCell<InvariantLifetime<'id>, T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}