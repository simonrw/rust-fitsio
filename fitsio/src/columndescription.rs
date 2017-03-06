/// Description for new columns
#[derive(Debug, Clone)]
pub struct ColumnDescription {
    pub name: String,

    // TODO: make this use one of the enums
    /// Type of the data, see the cfitsio documentation
    pub data_type: String,
}
