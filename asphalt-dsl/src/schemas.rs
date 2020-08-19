use crate::expressions::IsExpression;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ident {
    name: &'static str,
    schema: &'static str,
}

impl Ident {
    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn schema(self) -> &'static str {
        self.schema
    }
}

//
// table!(public.users {
//     pk user_id: Uuid,
//     fk tenant_id: Uuid -> public.tenant,
// })
// --
//
// mod users {
//     #[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
//     pub struct table;
//
//     impl IsTable for table {
//         const DESCRIPTION: &'static Table = &Table {
//             ident: Ident { name: "users", schema: "public" },
//             all_columns: &[Column {
//                 name: "user_id"
//             }],
//         };
//
//         type PrimaryKey = user_id;
//     }
//
//     #[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
//     pub struct user_id;
//
//     mod user_id {
//         impl HasTable for user_id {
//             type Table = table;
//         }
//
//         impl HasType for user_id {
//              type Type = Uuid;
//         }
//
//         impl IsColumn for user_id {
//             const DESCRIPTION: &'static Column = &table.all_columns[0];
//         }
//     }
// }
pub trait IsTable: Default {
    const DESCRIPTION: &'static Table;
    const COLUMNS: &'static [Column];

    type PrimaryKey: IsColumn;
    type AllColumns: AppearsOnTable<Self> + Default;
}

/// The primary key of the table.
pub type Pk<T> = <T as IsTable>::PrimaryKey;

/// All the columns of the table.
pub type AllColumns<T> = <T as IsTable>::AllColumns;

pub struct Table {
    pub ident: Ident,
    pub all_columns: &'static [Column],
}

pub trait HasTable {
    type Table: IsTable;
}

pub trait IsColumn: HasTable + IsExpression + Default {
    const DESCRIPTION: &'static Column;
}

pub struct Column {
    pub name: &'static str,
}

#[marker]
pub trait AppearsOnTable<T: IsTable>: IsExpression {}

/// Every column appears on its table.
impl<T, C> AppearsOnTable<T> for C
where
    T: IsTable,
    C: IsColumn<Table = T>,
{
}
