//! Contains the `Row` trait

use crate::backend::{self, Backend};
use crate::deserialize;
use deserialize::FromSql;
use std::ops::Range;

#[cfg(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes")]
#[doc(inline)]
pub use self::private::{PartialRow, RowGatWorkaround};

#[cfg(not(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes"))]
pub(crate) use self::private::{PartialRow, RowGatWorkaround};

/// Representing a way to index into database rows
///
/// * Crates using existing backends should use existing implementations of
///   this traits. Diesel provides `RowIndex<usize>` and `RowIndex<&str>` for
///   all built-in backends
///
/// * Crates implementing custom backends need to provide `RowIndex<usize>` and
///   `RowIndex<&str>` impls for their [`Row`] type.
///
pub trait RowIndex<I> {
    /// Get the numeric index inside the current row for the provided index value
    fn idx(&self, idx: I) -> Option<usize>;
}

/// Return type of [`Row::get`]
///
/// Users should threat this as opaque [`impl Field<DB>`](Field) type.
pub type FieldRet<'a, R, DB> = <R as RowGatWorkaround<'a, DB>>::Field;

/// Represents a single database row.
///
/// This trait is used as an argument to [`FromSqlRow`].
///
/// [`FromSqlRow`]: crate::deserialize::FromSqlRow
pub trait Row<'a, DB: Backend>:
    RowIndex<usize> + for<'b> RowIndex<&'b str> + for<'b> RowGatWorkaround<'b, DB> + Sized
{
    /// Return type of `PartialRow`
    ///
    /// For all implementations, beside of the `Row` implementation on `PartialRow` itself
    /// this should be `Self`.
    #[cfg_attr(
        not(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes"),
        doc(hidden)
    )]
    type InnerPartialRow: Row<'a, DB>;

    /// Get the number of fields in the current row
    fn field_count(&self) -> usize;

    /// Get the field with the provided index from the row.
    ///
    /// Returns `None` if there is no matching field for the given index
    fn get<'b, I>(&'b self, idx: I) -> Option<FieldRet<'b, Self, DB>>
    where
        'a: 'b,
        Self: RowIndex<I>;

    /// Get a deserialized value with the provided index from the row.
    fn get_value<ST, T, I>(&self, idx: I) -> crate::deserialize::Result<T>
    where
        Self: RowIndex<I>,
        T: FromSql<ST, DB>,
    {
        let field = self.get(idx).ok_or(crate::result::UnexpectedEndOfRow)?;
        <T as FromSql<ST, DB>>::from_nullable_sql(field.value())
    }

    /// Returns a wrapping row that allows only to access fields, where the index is part of
    /// the provided range.
    #[cfg_attr(
        not(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes"),
        doc(hidden)
    )]
    fn partial_row(&self, range: Range<usize>) -> PartialRow<'_, Self::InnerPartialRow>;
}

/// Represents a single field in a database row.
///
/// This trait allows retrieving information on the name of the colum and on the value of the
/// field.
pub trait Field<'a, DB: Backend> {
    /// The name of the current field
    ///
    /// Returns `None` if it's an unnamed field
    fn field_name(&self) -> Option<&str>;

    /// Get the value representing the current field in the raw representation
    /// as it is transmitted by the database
    fn value(&self) -> Option<backend::RawValue<'_, DB>>;

    /// Checks whether this field is null or not.
    fn is_null(&self) -> bool {
        self.value().is_none()
    }
}

impl<'a, 'b, DB, R> RowGatWorkaround<'a, DB> for PartialRow<'b, R>
where
    DB: Backend,
    R: RowGatWorkaround<'a, DB>,
{
    type Field = R::Field;
}
/// Represents a row of a SQL query, where the values are accessed by name
/// rather than by index.
///
/// This trait is used by implementations of
/// [`QueryableByName`](crate::deserialize::QueryableByName)
pub trait NamedRow<'a, DB: Backend>: Row<'a, DB> {
    /// Retrieve and deserialize a single value from the query
    ///
    /// Note that `ST` *must* be the exact type of the value with that name in
    /// the query. The compiler will not be able to verify that you have
    /// provided the correct type. If there is a mismatch, you may receive an
    /// incorrect value, or a runtime error.
    ///
    /// If two or more fields in the query have the given name, the result of
    /// this function is undefined.
    fn get<ST, T>(&self, column_name: &str) -> deserialize::Result<T>
    where
        T: FromSql<ST, DB>;
}

impl<'a, R, DB> NamedRow<'a, DB> for R
where
    R: Row<'a, DB>,
    DB: Backend,
{
    fn get<ST, T>(&self, column_name: &str) -> deserialize::Result<T>
    where
        T: FromSql<ST, DB>,
    {
        let field = Row::get(self, column_name)
            .ok_or_else(|| format!("Column `{}` was not present in query", column_name))?;

        T::from_nullable_sql(field.value())
    }
}

// These traits are not part of the public API
// because we want to replace them by with an associated type
// in the child trait later if GAT's are finally stable
mod private {
    use super::*;

    /// A helper trait to indicate the life time bound for a field returned
    /// by [`Row::get`]
    #[cfg_attr(
        feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes",
        cfg(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes")
    )]
    pub trait RowGatWorkaround<'a, DB: Backend> {
        /// Field type returned by a `Row` implementation
        ///
        /// * Crates using existing backend should not concern themself with the
        ///   concrete type of this associated type.
        ///
        /// * Crates implementing custom backends should provide their own type
        ///   meeting the required trait bounds
        type Field: Field<'a, DB>;
    }

    /// A row type that wraps an inner row
    ///
    /// This type only allows to access fields of the inner row, whose index is
    /// part of `range`. This type is used by diesel internally to implement
    /// [`FromStaticSqlRow`](crate::deserialize::FromStaticSqlRow).
    ///
    /// Indexing via `usize` starts with 0 for this row type. The index is then shifted
    /// by `self.range.start` to match the corresponding field in the underlying row.
    #[derive(Debug)]
    #[cfg_attr(
        feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes",
        cfg(feature = "i-implement-a-third-party-backend-and-opt-into-breaking-changes")
    )]
    pub struct PartialRow<'a, R> {
        inner: &'a R,
        range: Range<usize>,
    }

    impl<'a, R> PartialRow<'a, R> {
        /// Create a new [`PartialRow`] instance based on an inner
        /// row and a range of field that should be part of the constructed
        /// wrapper.
        ///
        /// See the documentation of [`PartialRow`] for details.
        pub fn new<'b, DB>(inner: &'a R, range: Range<usize>) -> Self
        where
            R: Row<'b, DB>,
            DB: Backend,
        {
            let range_lower = std::cmp::min(range.start, inner.field_count());
            let range_upper = std::cmp::min(range.end, inner.field_count());
            Self {
                inner,
                range: range_lower..range_upper,
            }
        }
    }

    impl<'a, 'b, DB, R> Row<'a, DB> for PartialRow<'b, R>
    where
        DB: Backend,
        R: Row<'a, DB>,
    {
        type InnerPartialRow = R;

        fn field_count(&self) -> usize {
            self.range.len()
        }

        fn get<'c, I>(&'c self, idx: I) -> Option<<Self as RowGatWorkaround<'c, DB>>::Field>
        where
            'a: 'c,
            Self: RowIndex<I>,
        {
            let idx = self.idx(idx)?;
            self.inner.get(idx)
        }

        fn partial_row(&self, range: Range<usize>) -> PartialRow<'_, R> {
            let range_upper_bound = std::cmp::min(self.range.end, self.range.start + range.end);
            let range = (self.range.start + range.start)..range_upper_bound;
            PartialRow {
                inner: self.inner,
                range,
            }
        }
    }

    impl<'a, 'b, R> RowIndex<&'a str> for PartialRow<'b, R>
    where
        R: RowIndex<&'a str>,
    {
        fn idx(&self, idx: &'a str) -> Option<usize> {
            let idx = self.inner.idx(idx)?;
            if self.range.contains(&idx) {
                Some(idx)
            } else {
                None
            }
        }
    }

    impl<'a, R> RowIndex<usize> for PartialRow<'a, R>
    where
        R: RowIndex<usize>,
    {
        fn idx(&self, idx: usize) -> Option<usize> {
            let idx = self.inner.idx(idx + self.range.start)?;
            if self.range.contains(&idx) {
                Some(idx)
            } else {
                None
            }
        }
    }
}
