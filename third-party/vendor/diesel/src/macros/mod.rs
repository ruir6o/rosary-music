#![allow(unused_parens)] // FIXME: Remove this attribute once false positive is resolved.
#![cfg_attr(rustfmt, rustfmt_skip)] // https://github.com/rust-lang-nursery/rustfmt/issues/2755

pub(crate) mod prelude {
    #[cfg_attr(
        any(feature = "huge-tables", feature = "large-tables"),
        allow(deprecated)
    )]
    // This is a false positive, we reexport it later
    #[allow(unreachable_pub)]
    #[doc(inline)]
    pub use crate::{
        allow_columns_to_appear_in_same_group_by_clause,
        allow_tables_to_appear_in_same_query,
        joinable,
        table,
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_fix_sql_type_import {
    ($(use $($import:tt)::+;)*) => {
        $(
            $crate::__diesel_fix_sql_type_import!(@expand_import: $($import)::+);
        )*
    };
    (@expand_import: super:: $($Type:tt)+) => {
        use super::super::$($Type)+;
    };
    (@expand_import: $($Type:tt)+) => {
        use $($Type)+;
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "postgres_backend")]
macro_rules! __diesel_internal_backend_specific_column_impls {
    ($table:ident, $column_name:ident) => {
        impl $crate::query_source::AppearsInFromClause<$crate::query_builder::Only<$table>>
            for $column_name {
            type Count = $crate::query_source::Once;
        }
        impl $crate::SelectableExpression<$crate::query_builder::Only<$table>> for $column_name {}
    }
}
#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "postgres_backend"))]
macro_rules! __diesel_internal_backend_specific_column_impls {
    ($table:ident, $column_name:ident) => {};
}
#[macro_export]
#[doc(hidden)]
#[cfg(feature = "postgres_backend")]
macro_rules! __diesel_internal_backend_specific_table_impls {
    ($table:ident) => {
        impl<S> $crate::JoinTo<$crate::query_builder::Only<S>> for $table
        where
            $crate::query_builder::Only<S>: $crate::JoinTo<$table>,
        {
            type FromClause = $crate::query_builder::Only<S>;
            type OnClause = <$crate::query_builder::Only<S> as $crate::JoinTo<$table>>::OnClause;

            fn join_target(__diesel_internal_rhs: $crate::query_builder::Only<S>) -> (Self::FromClause, Self::OnClause) {
                let (_, __diesel_internal_on_clause) = $crate::query_builder::Only::<S>::join_target($table);
                (__diesel_internal_rhs, __diesel_internal_on_clause)
            }
        }

        impl $crate::query_source::AppearsInFromClause<$crate::query_builder::Only<$table>>
            for $table {
            type Count = $crate::query_source::Once;
        }
        impl $crate::query_source::AppearsInFromClause<$table>
            for $crate::query_builder::Only<$table> {
            type Count = $crate::query_source::Once;
        }
    };
}
#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "postgres_backend"))]
macro_rules! __diesel_internal_backend_specific_table_impls {
    ($table:ident) => {};
}


#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_column {
    (
        table = $table:ident,
        table_sql_name = $table_sql_name:expr,
        table_schema = $table_schema:ident,
        name = $column_name:ident,
        sql_name = $sql_name:expr,
        ty = ($($Type:tt)*),
        meta = [$($meta:tt)*],
    ) => {
        $($meta)*
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Debug, Clone, Copy, $crate::query_builder::QueryId, Default)]
        pub struct $column_name;

        impl $crate::expression::Expression for $column_name {
            type SqlType = $($Type)*;
        }

        impl<DB> $crate::query_builder::QueryFragment<DB> for $column_name where
            DB: $crate::backend::Backend,
            $crate::internal::table_macro::StaticQueryFragmentInstance<table>: $crate::query_builder::QueryFragment<DB>,
        {
            #[allow(non_snake_case)]
            fn walk_ast<'b>(&'b self, mut __diesel_internal_out: $crate::query_builder::AstPass<'_, 'b, DB>) -> $crate::result::QueryResult<()>
            {
                if !__diesel_internal_out.should_skip_from() {
                    const FROM_CLAUSE: $crate::internal::table_macro::StaticQueryFragmentInstance<table> = $crate::internal::table_macro::StaticQueryFragmentInstance::new();

                    FROM_CLAUSE.walk_ast(__diesel_internal_out.reborrow())?;
                    __diesel_internal_out.push_sql(".");
                }
                __diesel_internal_out.push_identifier($sql_name)
            }
        }

        impl $crate::SelectableExpression<$table> for $column_name {
        }

        impl<QS> $crate::AppearsOnTable<QS> for $column_name where
            QS: $crate::query_source::AppearsInFromClause<$table, Count=$crate::query_source::Once>,
        {
        }

        impl<Left, Right> $crate::SelectableExpression<
                $crate::internal::table_macro::Join<Left, Right, $crate::internal::table_macro::LeftOuter>,
        > for $column_name where
            $column_name: $crate::AppearsOnTable<$crate::internal::table_macro::Join<Left, Right, $crate::internal::table_macro::LeftOuter>>,
            Self: $crate::SelectableExpression<Left>,
            // If our table is on the right side of this join, only
            // `Nullable<Self>` can be selected
            Right: $crate::query_source::AppearsInFromClause<$table, Count=$crate::query_source::Never> + $crate::query_source::QuerySource,
            Left: $crate::query_source::QuerySource
        {
        }

        impl<Left, Right> $crate::SelectableExpression<
                $crate::internal::table_macro::Join<Left, Right, $crate::internal::table_macro::Inner>,
        > for $column_name where
            $column_name: $crate::AppearsOnTable<$crate::internal::table_macro::Join<Left, Right, $crate::internal::table_macro::Inner>>,
            Left: $crate::query_source::AppearsInFromClause<$table> + $crate::query_source::QuerySource,
            Right: $crate::query_source::AppearsInFromClause<$table> + $crate::query_source::QuerySource,
            (Left::Count, Right::Count): $crate::internal::table_macro::Pick<Left, Right>,
            Self: $crate::SelectableExpression<
                <(Left::Count, Right::Count) as $crate::internal::table_macro::Pick<Left, Right>>::Selection,
            >,
        {
        }

        // FIXME: Remove this when overlapping marker traits are stable
        impl<Join, On> $crate::SelectableExpression<$crate::internal::table_macro::JoinOn<Join, On>> for $column_name where
            $column_name: $crate::SelectableExpression<Join> + $crate::AppearsOnTable<$crate::internal::table_macro::JoinOn<Join, On>>,
        {
        }

        // FIXME: Remove this when overlapping marker traits are stable
        impl<From> $crate::SelectableExpression<$crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<From>>> for $column_name where
            From: $crate::query_source::QuerySource,
            $column_name: $crate::SelectableExpression<From> + $crate::AppearsOnTable<$crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<From>>>,
        {
        }

        impl<__GB> $crate::expression::ValidGrouping<__GB> for $column_name
        where __GB: $crate::expression::IsContainedInGroupBy<$column_name, Output = $crate::expression::is_contained_in_group_by::Yes>,
        {
            type IsAggregate = $crate::expression::is_aggregate::Yes;
        }

        impl $crate::expression::ValidGrouping<()> for $column_name {
            type IsAggregate = $crate::expression::is_aggregate::No;
        }

        impl $crate::expression::IsContainedInGroupBy<$column_name> for $column_name {
            type Output = $crate::expression::is_contained_in_group_by::Yes;
        }

        impl $crate::query_source::Column for $column_name {
            type Table = $table;

            const NAME: &'static str = $sql_name;
        }

        impl<T> $crate::EqAll<T> for $column_name where
            T: $crate::expression::AsExpression<$($Type)*>,
            $crate::dsl::Eq<$column_name, T::Expression>: $crate::Expression<SqlType=$crate::sql_types::Bool>,
        {
            type Output = $crate::dsl::Eq<Self, T::Expression>;

            fn eq_all(self, __diesel_internal_rhs: T) -> Self::Output {
                use $crate::expression_methods::ExpressionMethods;
                self.eq(__diesel_internal_rhs)
            }
        }


        $crate::__diesel_generate_ops_impls_if_numeric!($column_name, $($Type)*);
        $crate::__diesel_generate_ops_impls_if_date_time!($column_name, $($Type)*);
        $crate::__diesel_generate_ops_impls_if_network!($column_name, $($Type)*);
        $crate::__diesel_internal_backend_specific_column_impls!($table, $column_name);
    }
}

/// Specifies that a table exists, and what columns it has. This will create a
/// new public module, with the same name, as the name of the table. In this
/// module, you'll find a unit struct named `table`, and a unit struct with the
/// names of each of the columns.
///
/// By default this allows a maximum of 32 columns per table.
/// You can increase this limit to 64 by enabling the `64-column-tables` feature.
/// You can increase it to 128 by enabling the `128-column-tables` feature.
/// You can decrease it to 16 columns,
/// which improves compilation time,
/// by disabling the default features of Diesel.
/// Note that enabling 64 column tables or larger will substantially increase
/// the compile time of Diesel.
///
/// Example usage
/// -------------
///
/// ```rust
/// diesel::table! {
///     users {
///         id -> Integer,
///         name -> VarChar,
///         favorite_color -> Nullable<VarChar>,
///     }
/// }
/// ```
///
/// You may also specify a primary key if it's called something other than `id`.
/// Tables with no primary key are not supported.
///
/// ```rust
/// diesel::table! {
///     users (non_standard_primary_key) {
///         non_standard_primary_key -> Integer,
///         name -> VarChar,
///         favorite_color -> Nullable<VarChar>,
///     }
/// }
/// ```
///
/// For tables with composite primary keys, list all of the columns in the
/// primary key.
///
/// ```rust
/// diesel::table! {
///     followings (user_id, post_id) {
///         user_id -> Integer,
///         post_id -> Integer,
///         favorited -> Bool,
///     }
/// }
/// # fn main() {
/// #     use diesel::prelude::Table;
/// #     use self::followings::dsl::*;
/// #     // Poor man's assert_eq! -- since this is type level this would fail
/// #     // to compile if the wrong primary key were generated
/// #     let (user_id {}, post_id {}) = followings.primary_key();
/// # }
/// ```
///
/// If you are using types that aren't from Diesel's core types, you can specify
/// which types to import.
///
/// ```
/// # mod diesel_full_text_search {
/// #     #[derive(diesel::sql_types::SqlType)]
/// #     pub struct TsVector;
/// # }
///
/// diesel::table! {
///     use diesel::sql_types::*;
/// #    use crate::diesel_full_text_search::*;
/// # /*
///     use diesel_full_text_search::*;
/// # */
///
///     posts {
///         id -> Integer,
///         title -> Text,
///         keywords -> TsVector,
///     }
/// }
/// # fn main() {}
/// ```
///
/// If you want to add documentation to the generated code you can use the
/// following syntax:
///
/// ```
/// diesel::table! {
///     /// The table containing all blog posts
///     posts {
///         /// The post's unique id
///         id -> Integer,
///         /// The post's title
///         title -> Text,
///     }
/// }
/// ```
///
/// If you have a column with the same name as a Rust reserved keyword, you can use
/// the `sql_name` attribute like this:
///
/// ```
/// diesel::table! {
///     posts {
///         id -> Integer,
///         /// This column is named `mytype` but references the table `type` column.
///         #[sql_name = "type"]
///         mytype -> Text,
///     }
/// }
/// ```
///
/// This module will also contain several helper types:
///
/// dsl
/// ---
///
/// This simply re-exports the table, renamed to the same name as the module,
/// and each of the columns. This is useful to glob import when you're dealing
/// primarily with one table, to allow writing `users.filter(name.eq("Sean"))`
/// instead of `users::table.filter(users::name.eq("Sean"))`.
///
/// `all_columns`
/// -----------
///
/// A constant will be assigned called `all_columns`. This is what will be
/// selected if you don't otherwise specify a select clause. It's type will be
/// `table::AllColumns`. You can also get this value from the
/// `Table::all_columns` function.
///
/// star
/// ----
///
/// This will be the qualified "star" expression for this table (e.g.
/// `users.*`). Internally, we read columns by index, not by name, so this
/// column is not safe to read data out of, and it has had it's SQL type set to
/// `()` to prevent accidentally using it as such. It is sometimes useful for
/// count statements however. It can also be accessed through the `Table.star()`
/// method.
///
/// `SqlType`
/// -------
///
/// A type alias called `SqlType` will be created. It will be the SQL type of
/// `all_columns`. The SQL type is needed for things like [returning boxed
/// queries][boxed_queries].
///
/// [boxed_queries]: crate::query_dsl::QueryDsl::into_boxed()
///
/// `BoxedQuery`
/// ----------
///
/// ```ignore
/// pub type BoxedQuery<'a, DB, ST = SqlType> = BoxedSelectStatement<'a, ST, table, DB>;
/// ```

#[allow(deprecated)]
//#[cfg_attr(feature="huge-tables", deprecated="`huge-tables` is deprecated in favor of `64-column-tables`")]
//#[cfg_attr(feature="large-tables", deprecated="`large-tables` is deprecated in favor of `32-column-tables`")]
#[macro_export]
macro_rules! table {
    ($($tokens:tt)*) => {
        $crate::__diesel_parse_table! {
            tokens = [$($tokens)*],
            imports = [],
            meta = [],
            sql_name = unknown,
            name = unknown,
            schema = public,
            primary_key = id,
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_invalid_table_syntax {
    () => {
        compile_error!(
            "Invalid `table!` syntax. Please see the `table!` macro docs for more info."
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_parse_table {
    // Found an import
    (
        tokens = [use $($import:tt)::+; $($rest:tt)*],
        imports = [$($imports:tt)*],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = [$($imports)* use $($import)::+;],
            $($args)*
        }
    };

    // Found sql_name attribute, override whatever we had before
    (
        tokens = [#[sql_name = $sql_name:expr] $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        sql_name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            sql_name = $sql_name,
            $($args)*
        }
    };

    // Meta item other than sql_name, attach it to the table struct
    (
        tokens = [#$new_meta:tt $($rest:tt)*],
        imports = $imports:tt,
        meta = [$($meta:tt)*],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = [$($meta)* #$new_meta],
            $($args)*
        }
    };

    // Found a schema name, override whatever we had before
    (
        tokens = [$schema:ident . $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        sql_name = $sql_name:tt,
        name = $name:tt,
        schema = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            sql_name = $sql_name,
            name = $name,
            schema = $schema,
            $($args)*
        }
    };

    // Found a table name, override whatever we had before
    (
        tokens = [$name:ident $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        sql_name = $sql_name:tt,
        name = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            sql_name = $sql_name,
            name = $name,
            $($args)*
        }
    };

    // Found a primary key, override whatever we had before
    (
        tokens = [($($pk:ident),+ $(,)*) $($rest:tt)*],
        imports = $imports:tt,
        meta = $meta:tt,
        sql_name = $sql_name:tt,
        name = $name:tt,
        schema = $schema:tt,
        primary_key = $ignore:tt,
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [$($rest)*],
            imports = $imports,
            meta = $meta,
            sql_name = $sql_name,
            name = $name,
            schema = $schema,
            primary_key = ($($pk),+),
            $($args)*
        }
    };

    // Reached columns with no imports, set a default
    (
        tokens = [{$($columns:tt)*}],
        imports = [],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [{$($columns)*}],
            imports = [use $crate::sql_types::*;],
            $($args)*
        }
    };

    // Reached columns with no sql_name, set a default
    (
        tokens = [{$($columns:tt)*}],
        imports = $imports:tt,
        meta = $meta:tt,
        sql_name = unknown,
        name = $name:tt,
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_table! {
            tokens = [{$($columns)*}],
            imports = $imports,
            meta = $meta,
            sql_name = stringify!($name),
            name = $name,
            $($args)*
        }
    };

    // Parse the columns
    (
        tokens = [{$($columns:tt)*}],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            tokens = [$($columns)*],
            table = { $($args)* },
            columns = [],
        }
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        $crate::__diesel_invalid_table_syntax!();
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_parse_columns {
    // No column being parsed, start a new one.
    // Attempt to capture the type as separate tokens if at all possible.
    (
        tokens = [
            $(#$meta:tt)*
            $name:ident -> $($ty:tt)::* $(<$($ty_params:tt)::*>)*,
            $($rest:tt)*
        ],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            current_column = {
                unchecked_meta = [$(#$meta)*],
                name = $name,
                sql_name = stringify!($name),
                ty = ($($ty)::* $(<$($ty_params)::*>)*),
                meta = [],
            },
            tokens = [$($rest)*],
            $($args)*
        }
    };

    // No column being parsed, start a new one. Couldn't keep the `ty` separate.
    (
        tokens = [
            $(#$meta:tt)*
            $name:ident -> $ty:ty,
            $($rest:tt)*
        ],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            current_column = {
                unchecked_meta = [$(#$meta)*],
                name = $name,
                sql_name = stringify!($name),
                ty = ($ty),
                meta = [],
            },
            tokens = [$($rest)*],
            $($args)*
        }
    };


    // Found #[sql_name]
    (
        current_column = {
            unchecked_meta = [#[sql_name = $sql_name:expr] $($meta:tt)*],
            name = $name:tt,
            sql_name = $ignore:expr,
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            current_column = {
                unchecked_meta = [$($meta)*],
                name = $name,
                sql_name = $sql_name,
                $($current_column)*
            },
            $($args)*
        }
    };

    // Meta item other than #[sql_name]
    (
        current_column = {
            unchecked_meta = [#$new_meta:tt $($unchecked_meta:tt)*],
            name = $name:tt,
            sql_name = $sql_name:expr,
            ty = $ty:tt,
            meta = [$($meta:tt)*],
            $($current_column:tt)*
        },
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            current_column = {
                unchecked_meta = [$($unchecked_meta)*],
                name = $name,
                sql_name = $sql_name,
                ty = $ty,
                meta = [$($meta)* #$new_meta],
                $($current_column)*
            },
            $($args)*
        }
    };

    // Done parsing this column
    (
        current_column = {
            unchecked_meta = [],
            $($current_column:tt)*
        },
        tokens = $tokens:tt,
        table = $table:tt,
        columns = [$($columns:tt,)*],
        $($args:tt)*
    ) => {
        $crate::__diesel_parse_columns! {
            tokens = $tokens,
            table = $table,
            columns = [$($columns,)* { $($current_column)* },],
            $($args)*
        }
    };

    // Done parsing all columns
    (
        tokens = [],
        $($args:tt)*
    ) => {
        $crate::__diesel_table_impl!($($args)*);
    };

    // Invalid syntax
    ($($tokens:tt)*) => {
        $crate::__diesel_invalid_table_syntax!();
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_table_impl {
    (
        table = { $($table: tt)* },
        columns = [$({
            name = $column_name:ident,
            sql_name = $column_sql_name:expr,
            ty = ($($column_ty:tt)*),
            $($column:tt)*
        },)+],
    ) => {
        $crate::__diesel_check_column_count!{
            inner = {
                $crate::__diesel_table_impl! {
                    impl_table,
                    table = { $($table)*},
                    columns = [$({
                        name = $column_name,
                        sql_name = $column_sql_name,
                        ty = ($($column_ty)*),
                        $($column)*
                    },)+],
                }
            },
            ($($column_name,)*)
        }
    };
    (
        impl_table,
        table = {
            imports = [$($imports:tt)*],
            meta = [$($meta:tt)*],
            sql_name = $sql_name:expr,
            name = $table_name:ident,
            schema = $schema:ident,
            primary_key = $primary_key:tt,
        },
        columns = [$({
            name = $column_name:ident,
            sql_name = $column_sql_name:expr,
            ty = ($($column_ty:tt)*),
            $($column:tt)*
        },)+],
    ) => {
        $($meta)*
        #[allow(unused_imports, dead_code, unreachable_pub)]
        pub mod $table_name {
            pub use self::columns::*;
            $($imports)*

            /// Re-exports all of the columns of this table, as well as the
            /// table struct renamed to the module name. This is meant to be
            /// glob imported for functions which only deal with one table.
            pub mod dsl {
                $($crate::static_cond! {
                    if $table_name == $column_name {
                        compile_error!(concat!(
                            "Column `",
                            stringify!($column_name),
                            "` cannot be named the same as its table.\n \
                            You may use `#[sql_name = \"",
                            stringify!($column_name),
                            "\"]` to reference the table's `",
                            stringify!($column_name),
                            "` column. \n \
                            Docs available at: `https://docs.diesel.rs/diesel/macro.table.html`\n"
                        ));
                    } else {
                        pub use super::columns::{$column_name};
                    }
                })+
                pub use super::table as $table_name;
            }

            #[allow(non_upper_case_globals, dead_code)]
            /// A tuple of all of the columns on this table
            pub const all_columns: ($($column_name,)+) = ($($column_name,)+);

            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, $crate::query_builder::QueryId)]
            /// The actual table struct
            ///
            /// This is the type which provides the base methods of the query
            /// builder, such as `.select` and `.filter`.
            pub struct table;

            impl table {
                #[allow(dead_code)]
                /// Represents `table_name.*`, which is sometimes necessary
                /// for efficient count queries. It cannot be used in place of
                /// `all_columns`
                pub fn star(&self) -> star {
                    star
                }
            }

            /// The SQL type of all of the columns on this table
            pub type SqlType = ($($($column_ty)*,)+);

            /// Helper type for representing a boxed query from this table
            pub type BoxedQuery<'a, DB, ST = SqlType> = $crate::internal::table_macro::BoxedSelectStatement<'a, ST, $crate::internal::table_macro::FromClause<table>, DB>;

            impl $crate::QuerySource for table {
                type FromClause = $crate::internal::table_macro::StaticQueryFragmentInstance<table>;
                type DefaultSelection = <Self as $crate::Table>::AllColumns;

                fn from_clause(&self) -> Self::FromClause {
                    $crate::internal::table_macro::StaticQueryFragmentInstance::new()
                }

                fn default_selection(&self) -> Self::DefaultSelection {
                    use $crate::Table;
                    Self::all_columns()
                }
            }

            impl<DB> $crate::query_builder::QueryFragment<DB> for table where
                DB: $crate::backend::Backend,
                <table as $crate::internal::table_macro::StaticQueryFragment>::Component: $crate::query_builder::QueryFragment<DB>
            {
                fn walk_ast<'b>(&'b self, __diesel_internal_pass: $crate::query_builder::AstPass<'_, 'b, DB>) -> $crate::result::QueryResult<()> {
                    <table as $crate::internal::table_macro::StaticQueryFragment>::STATIC_COMPONENT.walk_ast(__diesel_internal_pass)
                }
            }

            $crate::__diesel_table_generate_static_query_fragment_for_table!($schema, table, $sql_name);

            impl $crate::query_builder::AsQuery for table {
                type SqlType = SqlType;
                type Query = $crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<Self>>;

                fn as_query(self) -> Self::Query {
                    $crate::internal::table_macro::SelectStatement::simple(self)
                }
            }

            impl $crate::Table for table {
                type PrimaryKey = $primary_key;
                type AllColumns = ($($column_name,)+);

                fn primary_key(&self) -> Self::PrimaryKey {
                    $primary_key
                }

                fn all_columns() -> Self::AllColumns {
                    ($($column_name,)+)
                }
            }

            impl $crate::associations::HasTable for table {
                type Table = Self;

                fn table() -> Self::Table {
                    table
                }
            }

            impl $crate::query_builder::IntoUpdateTarget for table {
                type WhereClause = <<Self as $crate::query_builder::AsQuery>::Query as $crate::query_builder::IntoUpdateTarget>::WhereClause;

                fn into_update_target(self) -> $crate::query_builder::UpdateTarget<Self::Table, Self::WhereClause> {
                    use $crate::query_builder::AsQuery;
                    let q: $crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<table>> = self.as_query();
                    q.into_update_target()
                }
            }

            impl $crate::query_source::AppearsInFromClause<table> for table {
                type Count = $crate::query_source::Once;
            }

            // impl<S: AliasSource<Table=table>> AppearsInFromClause<table> for Alias<S>
            impl<S> $crate::internal::table_macro::AliasAppearsInFromClause<S, table> for table
            where S: $crate::query_source::AliasSource<Target=table>,
            {
                type Count = $crate::query_source::Never;
            }

            // impl<S1: AliasSource<Table=table>, S2: AliasSource<Table=table>> AppearsInFromClause<Alias<S1>> for Alias<S2>
            // Those are specified by the `alias!` macro, but this impl will allow it to implement this trait even in downstream
            // crates from the schema
            impl<S1, S2> $crate::internal::table_macro::AliasAliasAppearsInFromClause<table, S2, S1> for table
            where S1: $crate::query_source::AliasSource<Target=table>,
                  S2: $crate::query_source::AliasSource<Target=table>,
                  S1: $crate::internal::table_macro::AliasAliasAppearsInFromClauseSameTable<S2, table>,
            {
                type Count = <S1 as $crate::internal::table_macro::AliasAliasAppearsInFromClauseSameTable<S2, table>>::Count;
            }

            impl<S> $crate::query_source::AppearsInFromClause<$crate::query_source::Alias<S>> for table
            where S: $crate::query_source::AliasSource,
            {
                type Count = $crate::query_source::Never;
            }

            impl<S, C> $crate::internal::table_macro::FieldAliasMapperAssociatedTypesDisjointnessTrick<table, S, C> for table
            where
                S: $crate::query_source::AliasSource<Target = table> + ::std::clone::Clone,
                C: $crate::query_source::Column<Table = table>,
            {
                type Out = $crate::query_source::AliasedField<S, C>;

                fn map(__diesel_internal_column: C, __diesel_internal_alias: &$crate::query_source::Alias<S>) -> Self::Out {
                    __diesel_internal_alias.field(__diesel_internal_column)
                }
            }

            impl $crate::query_source::AppearsInFromClause<table> for $crate::internal::table_macro::NoFromClause {
                type Count = $crate::query_source::Never;
            }

            impl<Left, Right, Kind> $crate::JoinTo<$crate::internal::table_macro::Join<Left, Right, Kind>> for table where
                $crate::internal::table_macro::Join<Left, Right, Kind>: $crate::JoinTo<table>,
                Left: $crate::query_source::QuerySource,
                Right: $crate::query_source::QuerySource,
            {
                type FromClause = $crate::internal::table_macro::Join<Left, Right, Kind>;
                type OnClause = <$crate::internal::table_macro::Join<Left, Right, Kind> as $crate::JoinTo<table>>::OnClause;

                fn join_target(__diesel_internal_rhs: $crate::internal::table_macro::Join<Left, Right, Kind>) -> (Self::FromClause, Self::OnClause) {
                    let (_, __diesel_internal_on_clause) = $crate::internal::table_macro::Join::join_target(table);
                    (__diesel_internal_rhs, __diesel_internal_on_clause)
                }
            }

            impl<Join, On> $crate::JoinTo<$crate::internal::table_macro::JoinOn<Join, On>> for table where
                $crate::internal::table_macro::JoinOn<Join, On>: $crate::JoinTo<table>,
            {
                type FromClause = $crate::internal::table_macro::JoinOn<Join, On>;
                type OnClause = <$crate::internal::table_macro::JoinOn<Join, On> as $crate::JoinTo<table>>::OnClause;

                fn join_target(__diesel_internal_rhs: $crate::internal::table_macro::JoinOn<Join, On>) -> (Self::FromClause, Self::OnClause) {
                    let (_, __diesel_internal_on_clause) = $crate::internal::table_macro::JoinOn::join_target(table);
                    (__diesel_internal_rhs, __diesel_internal_on_clause)
                }
            }

            impl<F, S, D, W, O, L, Of, G> $crate::JoinTo<$crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<F>, S, D, W, O, L, Of, G>> for table where
                $crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<F>, S, D, W, O, L, Of, G>: $crate::JoinTo<table>,
                F: $crate::query_source::QuerySource
            {
                type FromClause = $crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<F>, S, D, W, O, L, Of, G>;
                type OnClause = <$crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<F>, S, D, W, O, L, Of, G> as $crate::JoinTo<table>>::OnClause;

                fn join_target(__diesel_internal_rhs: $crate::internal::table_macro::SelectStatement<$crate::internal::table_macro::FromClause<F>, S, D, W, O, L, Of, G>) -> (Self::FromClause, Self::OnClause) {
                    let (_, __diesel_internal_on_clause) = $crate::internal::table_macro::SelectStatement::join_target(table);
                    (__diesel_internal_rhs, __diesel_internal_on_clause)
                }
            }

            impl<'a, QS, ST, DB> $crate::JoinTo<$crate::internal::table_macro::BoxedSelectStatement<'a, $crate::internal::table_macro::FromClause<QS>, ST, DB>> for table where
                $crate::internal::table_macro::BoxedSelectStatement<'a, $crate::internal::table_macro::FromClause<QS>, ST, DB>: $crate::JoinTo<table>,
                QS: $crate::query_source::QuerySource,
            {
                type FromClause = $crate::internal::table_macro::BoxedSelectStatement<'a, $crate::internal::table_macro::FromClause<QS>, ST, DB>;
                type OnClause = <$crate::internal::table_macro::BoxedSelectStatement<'a, $crate::internal::table_macro::FromClause<QS>, ST, DB> as $crate::JoinTo<table>>::OnClause;
                fn join_target(__diesel_internal_rhs: $crate::internal::table_macro::BoxedSelectStatement<'a, $crate::internal::table_macro::FromClause<QS>, ST, DB>) -> (Self::FromClause, Self::OnClause) {
                    let (_, __diesel_internal_on_clause) = $crate::internal::table_macro::BoxedSelectStatement::join_target(table);
                    (__diesel_internal_rhs, __diesel_internal_on_clause)
                }
            }

            impl<S> $crate::JoinTo<$crate::query_source::Alias<S>> for table
            where
                $crate::query_source::Alias<S>: $crate::JoinTo<table>,
            {
                type FromClause = $crate::query_source::Alias<S>;
                type OnClause = <$crate::query_source::Alias<S> as $crate::JoinTo<table>>::OnClause;

                fn join_target(__diesel_internal_rhs: $crate::query_source::Alias<S>) -> (Self::FromClause, Self::OnClause) {
                    let (_, __diesel_internal_on_clause) = $crate::query_source::Alias::<S>::join_target(table);
                    (__diesel_internal_rhs, __diesel_internal_on_clause)
                }
            }

            // This impl should be able to live in Diesel,
            // but Rust tries to recurse for no reason
            impl<T> $crate::insertable::Insertable<T> for table
            where
                <table as $crate::query_builder::AsQuery>::Query: $crate::insertable::Insertable<T>,
            {
                type Values = <<table as $crate::query_builder::AsQuery>::Query as $crate::insertable::Insertable<T>>::Values;

                fn values(self) -> Self::Values {
                    use $crate::query_builder::AsQuery;
                    self.as_query().values()
                }
            }

            impl<'a, T> $crate::insertable::Insertable<T> for &'a table
            where
                table: $crate::insertable::Insertable<T>,
            {
                type Values = <table as $crate::insertable::Insertable<T>>::Values;

                fn values(self) -> Self::Values {
                    (*self).values()
                }
            }

            $crate::__diesel_internal_backend_specific_table_impls!(table);

            /// Contains all of the columns of this table
            pub mod columns {
                use super::table;
                $crate::__diesel_fix_sql_type_import!($($imports)*);

                #[allow(non_camel_case_types, dead_code)]
                #[derive(Debug, Clone, Copy, $crate::query_builder::QueryId)]
                /// Represents `table_name.*`, which is sometimes needed for
                /// efficient count queries. It cannot be used in place of
                /// `all_columns`, and has a `SqlType` of `()` to prevent it
                /// being used that way
                pub struct star;

                impl<__GB> $crate::expression::ValidGrouping<__GB> for star
                where
                    ($($column_name,)+): $crate::expression::ValidGrouping<__GB>,
                {
                    type IsAggregate = <($($column_name,)+) as $crate::expression::ValidGrouping<__GB>>::IsAggregate;
                }

                impl $crate::Expression for star {
                    type SqlType = $crate::expression::expression_types::NotSelectable;
                }

                impl<DB: $crate::backend::Backend> $crate::query_builder::QueryFragment<DB> for star where
                    <table as $crate::QuerySource>::FromClause: $crate::query_builder::QueryFragment<DB>,
                {
                    #[allow(non_snake_case)]
                    fn walk_ast<'b>(&'b self, mut __diesel_internal_out: $crate::query_builder::AstPass<'_, 'b, DB>) -> $crate::result::QueryResult<()>
                    {
                        use $crate::QuerySource;

                        if !__diesel_internal_out.should_skip_from() {
                            const FROM_CLAUSE: $crate::internal::table_macro::StaticQueryFragmentInstance<table> = $crate::internal::table_macro::StaticQueryFragmentInstance::new();

                            FROM_CLAUSE.walk_ast(__diesel_internal_out.reborrow())?;
                            __diesel_internal_out.push_sql(".");
                        }
                        __diesel_internal_out.push_sql("*");
                        Ok(())
                    }
                }

                impl $crate::SelectableExpression<table> for star {
                }

                impl $crate::AppearsOnTable<table> for star {
                }

                $($crate::__diesel_column! {
                    table = table,
                    table_sql_name = $sql_name,
                    table_schema = $schema,
                    name = $column_name,
                    sql_name = $column_sql_name,
                    ty = ($($column_ty)*),
                    $($column)*
                })+

                $crate::__diesel_valid_grouping_for_table_columns! {
                    primary_key = $primary_key, $($column_name,)*
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_valid_grouping_for_table_columns {
    (primary_key = ($primary_key: tt), $($cols: ident,)* ) => {
        $crate::__diesel_valid_grouping_for_table_columns! {
            primary_key = $primary_key,
            $($cols,)*
        }
    };
    (primary_key = $primary_key: tt, $left_col: ident, $($right_col: ident,)+) => {
        $(
            $crate::static_cond! {
                if $left_col == $primary_key {
                    impl $crate::expression::IsContainedInGroupBy<$right_col> for $left_col {
                        type Output = $crate::expression::is_contained_in_group_by::Yes;
                    }
                } else {
                    impl $crate::expression::IsContainedInGroupBy<$right_col> for $left_col {
                        type Output = $crate::expression::is_contained_in_group_by::No;
                    }
                }
            }

            $crate::static_cond! {
                if $right_col == $primary_key {
                    impl $crate::expression::IsContainedInGroupBy<$left_col> for $right_col {
                        type Output = $crate::expression::is_contained_in_group_by::Yes;
                    }
                } else {
                    impl $crate::expression::IsContainedInGroupBy<$left_col> for $right_col {
                        type Output = $crate::expression::is_contained_in_group_by::No;
                    }
                }
            }
       )*
        $crate::__diesel_valid_grouping_for_table_columns! {
            primary_key = $primary_key,
            $($right_col,)*
        }
    };
    (primary_key = $primary_key: tt, $left_col: ident,) => {};
    (primary_key = $primary_key: tt) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_table_generate_static_query_fragment_for_table {
    (public, $table: ident, $table_name:expr) => {
        impl $crate::internal::table_macro::StaticQueryFragment for table {
            type Component = $crate::internal::table_macro::Identifier<'static>;
            const STATIC_COMPONENT: &'static Self::Component = &$crate::internal::table_macro::Identifier($table_name);
        }

    };
    ($schema_name:ident, $table: ident, $table_name:expr) => {
        impl $crate::internal::table_macro::StaticQueryFragment for table {
            type Component = $crate::internal::table_macro::InfixNode<
                    $crate::internal::table_macro::Identifier<'static>,
                    $crate::internal::table_macro::Identifier<'static>,
                    &'static str
                >;
            const STATIC_COMPONENT: &'static Self::Component = &$crate::internal::table_macro::InfixNode::new(
                $crate::internal::table_macro::Identifier(stringify!($schema_name)),
                $crate::internal::table_macro::Identifier($table_name),
                "."
            );
        }
    }
}





/// Allow two tables to be referenced in a join query without providing an
/// explicit `ON` clause.
///
/// The generated `ON` clause will always join to the primary key of the parent
/// table. This macro removes the need to call [`.on`] explicitly, you will
/// still need to invoke
/// [`allow_tables_to_appear_in_same_query!`](crate::allow_tables_to_appear_in_same_query)
/// for these two tables to be able to use the resulting query, unless you are
/// using `diesel print-schema` which will generate it for you.
///
/// If you are using `diesel print-schema`, an invocation of this macro
/// will be generated for every foreign key in your database unless
/// one of the following is true:
///
/// - The foreign key references something other than the primary key
/// - The foreign key is composite
/// - There is more than one foreign key connecting two tables
/// - The foreign key is self-referential
///
/// # Example
///
/// ```rust
/// # include!("../doctest_setup.rs");
/// use schema::*;
///
/// # /*
/// joinable!(posts -> users (user_id));
/// allow_tables_to_appear_in_same_query!(posts, users);
/// # */
///
/// # fn main() {
/// let implicit_on_clause = users::table.inner_join(posts::table);
/// let implicit_on_clause_sql = diesel::debug_query::<DB, _>(&implicit_on_clause).to_string();
///
/// let explicit_on_clause = users::table
///     .inner_join(posts::table.on(posts::user_id.eq(users::id)));
/// let explicit_on_clause_sql = diesel::debug_query::<DB, _>(&explicit_on_clause).to_string();
///
/// assert_eq!(implicit_on_clause_sql, explicit_on_clause_sql);
/// # }
///
/// ```
///
/// In the example above, the line `joinable!(posts -> users (user_id));`
///
/// specifies the relation of the tables and the ON clause in the following way:
///
/// `child_table -> parent_table (foreign_key)`
///
/// * `parent_table` is the Table with the Primary key.
///
/// * `child_table` is the Table with the Foreign key.
///
/// So given the Table decaration from [Associations docs](crate::associations)
///
/// * The parent table would be `User`
/// * The child table would be `Post`
/// * and the Foreign key would be `Post.user_id`
///
/// For joins that do not explicitly use on clauses via [`JoinOnDsl`](crate::prelude::JoinOnDsl)
/// the following on clause is generated implicitly:
/// ```sql
/// post JOIN users ON posts.user_id = users.id
/// ```
#[macro_export]
macro_rules! joinable {
    ($($child:ident)::* -> $($parent:ident)::* ($source:ident)) => {
        $crate::joinable_inner!($($child)::* ::table => $($parent)::* ::table : ($($child)::* ::$source = $($parent)::* ::table));
        $crate::joinable_inner!($($parent)::* ::table => $($child)::* ::table : ($($child)::* ::$source = $($parent)::* ::table));
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! joinable_inner {
    ($left_table:path => $right_table:path : ($foreign_key:path = $parent_table:path)) => {
        $crate::joinable_inner!(
            left_table_ty = $left_table,
            right_table_ty = $right_table,
            right_table_expr = $right_table,
            foreign_key = $foreign_key,
            primary_key_ty = <$parent_table as $crate::query_source::Table>::PrimaryKey,
            primary_key_expr =
                <$parent_table as $crate::query_source::Table>::primary_key(&$parent_table),
        );
    };

    (
        left_table_ty = $left_table_ty:ty,
        right_table_ty = $right_table_ty:ty,
        right_table_expr = $right_table_expr:expr,
        foreign_key = $foreign_key:path,
        primary_key_ty = $primary_key_ty:ty,
        primary_key_expr = $primary_key_expr:expr,
    ) => {
        impl $crate::JoinTo<$right_table_ty> for $left_table_ty {
            type FromClause = $right_table_ty;
            type OnClause = $crate::dsl::Eq<
                $crate::internal::table_macro::NullableExpression<$foreign_key>,
                $crate::internal::table_macro::NullableExpression<$primary_key_ty>,
            >;

            fn join_target(rhs: $right_table_ty) -> (Self::FromClause, Self::OnClause) {
                use $crate::{ExpressionMethods, NullableExpressionMethods};

                (
                    rhs,
                    $foreign_key.nullable().eq($primary_key_expr.nullable()),
                )
            }
        }
    };
}

/// Allow two or more tables which are otherwise unrelated to be used together
/// in a query.
///
/// This macro must be invoked any time two tables need to appear in the same
/// query either because they are being joined together, or because one appears
/// in a subselect. When this macro is invoked with more than 2 tables, every
/// combination of those tables will be allowed to appear together.
///
/// If you are using `diesel print-schema`, an invocation of
/// this macro will be generated for you for all tables in your schema.
///
/// # Example
///
/// ```
/// # use diesel::{allow_tables_to_appear_in_same_query, table};
/// #
/// // This would be required to do `users.inner_join(posts.inner_join(comments))`
/// allow_tables_to_appear_in_same_query!(comments, posts, users);
///
/// table! {
///     comments {
///         id -> Integer,
///         post_id -> Integer,
///         body -> VarChar,
///     }
/// }
///
/// table! {
///    posts {
///        id -> Integer,
///        user_id -> Integer,
///        title -> VarChar,
///    }
/// }
///
/// table! {
///     users {
///        id -> Integer,
///        name -> VarChar,
///     }
/// }
/// ```
///
/// When more than two tables are passed, the relevant code is generated for
/// every combination of those tables. This code would be equivalent to the
/// previous example.
///
/// ```
/// # use diesel::{allow_tables_to_appear_in_same_query, table};
/// # table! {
/// #    comments {
/// #        id -> Integer,
/// #        post_id -> Integer,
/// #        body -> VarChar,
/// #    }
/// # }
/// #
/// # table! {
/// #    posts {
/// #        id -> Integer,
/// #        user_id -> Integer,
/// #        title -> VarChar,
/// #    }
/// # }
/// #
/// # table! {
/// #     users {
/// #        id -> Integer,
/// #        name -> VarChar,
/// #     }
/// # }
/// #
/// allow_tables_to_appear_in_same_query!(comments, posts);
/// allow_tables_to_appear_in_same_query!(comments, users);
/// allow_tables_to_appear_in_same_query!(posts, users);
/// #
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! allow_tables_to_appear_in_same_query {
    ($left_mod:ident, $($right_mod:ident),+ $(,)*) => {
        $(
            impl $crate::query_source::TableNotEqual<$left_mod::table> for $right_mod::table {}
            impl $crate::query_source::TableNotEqual<$right_mod::table> for $left_mod::table {}
            $crate::__diesel_internal_backend_specific_allow_tables_to_appear_in_same_query!($left_mod, $right_mod);
        )+
        $crate::allow_tables_to_appear_in_same_query!($($right_mod,)+);
    };

    ($last_table:ident,) => {};

    () => {};
}
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "postgres_backend")]
macro_rules! __diesel_internal_backend_specific_allow_tables_to_appear_in_same_query {
    ($left:ident, $right:ident) => {
        impl $crate::query_source::TableNotEqual<$left::table> for $crate::query_builder::Only<$right::table> {}
        impl $crate::query_source::TableNotEqual<$right::table> for $crate::query_builder::Only<$left::table> {}
        impl $crate::query_source::TableNotEqual<$crate::query_builder::Only<$left::table>>
            for $right::table {}
        impl $crate::query_source::TableNotEqual<$crate::query_builder::Only<$right::table>>
            for $left::table {}
    }
}
#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "postgres_backend"))]
macro_rules! __diesel_internal_backend_specific_allow_tables_to_appear_in_same_query {
    ($left:ident, $rigth:ident) => { }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __diesel_impl_allow_in_same_group_by_clause {
    (
        left = [$($left_path:tt)::+],
    ) => {};
    (
        left = [$($left_path:tt)::+],
        $($right_path:tt)::+
    ) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)+],
            right = [$($right_path)+],
            left_tbl = [],
            left_path = [],
        }
    };
    (
        left = [$($left_path:tt)::+],
        $($right_path:tt)::+,
        $($other:tt)*
    ) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)+],
            right = [$($right_path)+],
            left_tbl = [],
            left_path = [],
        }
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)::+],
            $($other)*
        }
    };
    (
        left = [$left_path_p1: tt  $($left_path: tt)+],
        right = [$($right_path: tt)*],
        left_tbl = [$($left_tbl:tt)?],
        left_path = [$($left_out_path:tt)*],
    ) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)+],
            right = [$($right_path)*],
            left_tbl = [$left_path_p1],
            left_path = [$($left_out_path)* $($left_tbl)?],
        }
    };
    (
        left = [$left_col: tt],
        right = [$($right_path: tt)*],
        left_tbl = [$($left_tbl:tt)?],
        left_path = [$($left_out_path:tt)*],
    ) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$left_col],
            right = [$($right_path)*],
            left_tbl = [$($left_tbl)?],
            left_path = [$($left_out_path)*],
            right_tbl = [],
            right_path = [],
        }
    };
    (
        left = [$left_col: tt ],
        right = [$right_path_p1: tt  $($right_path: tt)+],
        left_tbl = [$($left_tbl:tt)?],
        left_path = [$($left_out_path:tt)*],
        right_tbl = [$($right_tbl:tt)?],
        right_path = [$($right_out_path:tt)*],
    ) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$left_col],
            right = [$($right_path)+],
            left_tbl = [$($left_tbl)?],
            left_path = [$($left_out_path)*],
            right_tbl = [$right_path_p1],
            right_path = [$($right_out_path)* $($right_tbl)?],
        }
    };
    (
        left = [$left_col: tt],
        right = [$right_col: tt],
        left_tbl = [$left_tbl:tt],
        left_path = [$($left_begin:tt)*],
        right_tbl = [$right_tbl:tt],
        right_path = [$($right_begin:tt)*],
    ) => {
        $crate::static_cond! {
            if $left_tbl != $right_tbl {
                impl $crate::expression::IsContainedInGroupBy<$($left_begin ::)* $left_tbl :: $left_col> for $($right_begin ::)* $right_tbl :: $right_col {
                    type Output = $crate::expression::is_contained_in_group_by::No;
                }

                impl $crate::expression::IsContainedInGroupBy<$($right_begin ::)* $right_tbl :: $right_col> for $($left_begin ::)* $left_tbl :: $left_col {
                    type Output = $crate::expression::is_contained_in_group_by::No;
                }
            }
        }
    };
    (
        left = [$left_col: tt],
        right = [$right_col: tt],
        left_tbl = [$($left_tbl:tt)?],
        left_path = [$($left_begin:tt)*],
        right_tbl = [$($right_tbl:tt)?],
        right_path = [$($right_begin:tt)*],
    ) => {
        impl $crate::expression::IsContainedInGroupBy<$($left_begin ::)* $($left_tbl ::)? $left_col> for $($right_begin ::)* $($right_tbl ::)? $right_col {
            type Output = $crate::expression::is_contained_in_group_by::No;
        }

        impl $crate::expression::IsContainedInGroupBy<$($right_begin ::)* $($right_tbl ::)? $right_col> for $($left_begin ::)* $($left_tbl ::)? $left_col {
            type Output = $crate::expression::is_contained_in_group_by::No;
        }
    };

}


/// Allow two or more columns which are otherwise unrelated to be used together
/// in a group by clause.
///
/// This macro must be invoked any time two columns need to appear in the same
/// group by clause. When this macro is invoked with more than 2 columns, every
/// combination of those columns will be allowed to appear together.
///
/// # Example
///
/// ```
/// # include!("../doctest_setup.rs");
/// # use crate::schema::{users, posts};
/// // This would be required
///
/// allow_columns_to_appear_in_same_group_by_clause!(users::name, posts::id, posts::title);
/// # fn main() {
/// // to do implement the following join
/// users::table.inner_join(posts::table).group_by((users::name, posts::id, posts::title))
/// # ;
/// # }
/// ```
///
/// When more than two columns are passed, the relevant code is generated for
/// every combination of those columns. This code would be equivalent to the
/// previous example.
///
/// ```
/// # include!("../doctest_setup.rs");
/// # use crate::schema::{users, posts};
/// #
/// allow_columns_to_appear_in_same_group_by_clause!(users::name, posts::title);
/// allow_columns_to_appear_in_same_group_by_clause!(users::name, posts::id);
/// allow_columns_to_appear_in_same_group_by_clause!(posts::title, posts::id);
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! allow_columns_to_appear_in_same_group_by_clause {
    ($($left_path:tt)::+, $($right_path:tt)::+ $(,)?) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)::+],
            $($right_path)::+,
        }
    };
    ($($left_path:tt)::+, $($right_path:tt)::+, $($other: tt)*) => {
        $crate::__diesel_impl_allow_in_same_group_by_clause! {
            left = [$($left_path)::+],
            $($right_path)::+,
            $($other)*
        }
        $crate::allow_columns_to_appear_in_same_group_by_clause! {
            $($right_path)::+,
            $($other)*
        }
    };
    ($last_col:ty,) => {};
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}


macro_rules! impl_column_check {
    ($(
        $Tuple:tt {
            $(($idx:tt) -> $T:ident, $ST:ident, $TT:ident,)+
        }
    )+) => {
        __diesel_with_dollar_sign! {
            ($d:tt) => {
                #[macro_export]
                #[doc(hidden)]
                macro_rules! __diesel_check_column_count_internal {
                    $(
                    (inner = {$d($d inner: tt)*}, ($($d $T: ident,)*)) => {
                        $d($d inner)*
                    };
                    )*
                    (inner = {$d($d inner: tt)*}, ($d($d names: ident,)*)) => {
                        $crate::__diesel_error_table_size!();
                    }
                }
            }
        }
    }
}

diesel_derives::__diesel_for_each_tuple!(impl_column_check);

#[cfg(not(any(
    feature = "32-column-tables",
    feature = "64-column-tables",
    feature = "128-column-tables"
)))]
#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_error_table_size {
    () => {
        compile_error!(
            "Table contains more than 16 columns. Consider enabling the `32-column-tables` feature to enable diesels support for tables with more than 16 columns."
        );

    }
}


#[cfg(all(
    feature = "32-column-tables",
    not(feature = "64-column-tables"),
    not(feature = "128-column-tables")
))]
#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_error_table_size {
    () => {
        compile_error!(
            "Table contains more than 32 columns. Consider enabling the `64-column-tables` feature to enable diesels support for tables with more than 32 columns."
        );

    }
}


#[cfg(all(
    feature = "32-column-tables",
    feature = "64-column-tables",
    not(feature = "128-column-tables")
))]
#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_error_table_size {
    () => {
        compile_error!(
            "Table contains more than 64 columns. Consider enabling the `128-column-tables` feature to enable diesels support for tables with more than 64 columns."
        );

    }
}


#[cfg(all(
    feature = "32-column-tables",
    feature = "64-column-tables",
    feature = "128-column-tables"
))]
#[macro_export]
#[doc(hidden)]
macro_rules! __diesel_error_table_size {
    () => {
        compile_error!(
            "You reached the end. Diesel does not support tables with more than 128 columns. Consider using less columns."
        );
    }
}



// The order of these modules is important (at least for those which have tests).
// Utility macros which don't call any others need to come first.
#[macro_use]
mod internal;
#[macro_use]
mod static_cond;
#[macro_use]
mod ops;


#[cfg(test)]
mod tests {
    use crate::prelude::*;

    table! {
        foo.bars {
            id -> Integer,
            baz -> Text,
        }
    }

    mod my_types {
        #[derive(Debug, Clone, Copy, crate::sql_types::SqlType)]
        pub struct MyCustomType;
    }

    table! {
        use crate::sql_types::*;
        use crate::macros::tests::my_types::*;

        table_with_custom_types {
            id -> Integer,
            my_type -> MyCustomType,
        }
    }

    table! {
        use crate::sql_types::*;
        use crate::macros::tests::my_types::*;

        /// Table documentation
        ///
        /// some in detail documentation
        table_with_custom_type_and_id (a) {
            /// Column documentation
            ///
            /// some more details
            a -> Integer,
            my_type -> MyCustomType,
        }
    }

    #[test]
    #[cfg(feature = "postgres")]
    fn table_with_custom_schema() {
        use crate::pg::Pg;
        let expected_sql = r#"SELECT "foo"."bars"."baz" FROM "foo"."bars" -- binds: []"#;
        assert_eq!(
            expected_sql,
            &crate::debug_query::<Pg, _>(&bars::table.select(bars::baz)).to_string()
        );
    }

    table! {
        use crate::sql_types;
        use crate::sql_types::*;

        table_with_arbitrarily_complex_types {
            id -> sql_types::Integer,
            qualified_nullable -> sql_types::Nullable<sql_types::Integer>,
            deeply_nested_type -> Nullable<Nullable<Integer>>,
            // This actually should work, but there appears to be a rustc bug
            // on the `AsExpression` bound for `EqAll` when the ty param is a projection
            // projected_type -> <Nullable<Integer> as sql_types::IntoNullable>::Nullable,
            //random_tuple -> (Integer, Integer),
        }
    }

    table!(
        foo {
            /// Column doc
            id -> Integer,

            #[sql_name = "type"]
            /// Also important to document this column
            mytype -> Integer,

            /// And this one
            #[sql_name = "bleh"]
            hey -> Integer,
        }
    );

    #[test]
    #[cfg(feature = "postgres")]
    fn table_with_column_renaming_postgres() {
        use crate::pg::Pg;
        let expected_sql =
            r#"SELECT "foo"."id", "foo"."type", "foo"."bleh" FROM "foo" WHERE ("foo"."type" = $1) -- binds: [1]"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Pg, _>(&foo::table.filter(foo::mytype.eq(1))).to_string()
        );
    }

    #[test]
    #[cfg(feature = "mysql")]
    fn table_with_column_renaming_mysql() {
        use crate::mysql::Mysql;
        let expected_sql =
            r#"SELECT `foo`.`id`, `foo`.`type`, `foo`.`bleh` FROM `foo` WHERE (`foo`.`type` = ?) -- binds: [1]"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Mysql, _>(&foo::table.filter(foo::mytype.eq(1))).to_string()
        );
    }

    #[test]
    #[cfg(feature = "sqlite")]
    fn table_with_column_renaming_sqlite() {
        use crate::sqlite::Sqlite;
        let expected_sql =
            r#"SELECT `foo`.`id`, `foo`.`type`, `foo`.`bleh` FROM `foo` WHERE (`foo`.`type` = ?) -- binds: [1]"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Sqlite, _>(&foo::table.filter(foo::mytype.eq(1))).to_string()
        );
    }

    table!(
        use crate::sql_types::*;

        /// Some documentation
        #[sql_name="mod"]
        /// Some more documentation
        bar {
            id -> Integer,
        }
    );

    #[test]
    #[cfg(feature = "postgres")]
    fn table_renaming_postgres() {
        use crate::pg::Pg;
        let expected_sql = r#"SELECT "mod"."id" FROM "mod" -- binds: []"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Pg, _>(&bar::table.select(bar::id)).to_string()
        );
    }

    #[test]
    #[cfg(feature = "mysql")]
    fn table_renaming_mysql() {
        use crate::mysql::Mysql;
        let expected_sql = r#"SELECT `mod`.`id` FROM `mod` -- binds: []"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Mysql, _>(&bar::table.select(bar::id)).to_string()
        );
    }

    #[test]
    #[cfg(feature = "sqlite")]
    fn table_renaming_sqlite() {
        use crate::sqlite::Sqlite;
        let expected_sql = r#"SELECT `mod`.`id` FROM `mod` -- binds: []"#;
        assert_eq!(
            expected_sql,
            crate::debug_query::<Sqlite, _>(&bar::table.select(bar::id)).to_string()
        );
    }

    mod tests_for_allow_combined_group_by_syntax {
        table! {
            a(b) {
                b -> Text,
                c -> Text,
                d -> Text,
                e -> Text,
            }
        }

        table! {
            b(a) {
                a -> Text,
                c -> Text,
                d -> Text,
            }
        }

        table! {
            c(a) {
                a -> Text,
                b -> Text,
                d -> Text,
            }
        }

        // allow using table::collumn
        allow_columns_to_appear_in_same_group_by_clause!(
            a::b, b::a, a::d,
        );

        // allow using full paths
        allow_columns_to_appear_in_same_group_by_clause!(
            self::a::c, self::b::c, self::b::d,
        );

        use self::a::d as a_d;
        use self::b::d as b_d;
        use self::c::d as c_d;

        // allow using plain identifiers
        allow_columns_to_appear_in_same_group_by_clause!(
            a_d, b_d, c_d
        );

        // allow mixing all variants
        allow_columns_to_appear_in_same_group_by_clause!(
            c_d, self::b::a, a::e,
        );
    }
}
