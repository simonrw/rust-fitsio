//! Data descriptions
//!
//! Images are described as [`ImageDescription`](struct.ImageDescription.html) objects.
//!
//! Columns are represented as
//! [`ConcreteColumnDescription`](struct.ConcreteColumnDescription.html). This is constructed
//! through the builder pattern, by creating a [`ColumnDescription`](struct.ColumnDescription.html)
//! and calling [`create`](struct.ColumnDescription.html#method.create)

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parsing() {
        let s = "1E";
        assert_eq!(
            s.parse::<ColumnDataDescription>().unwrap(),
            ColumnDataDescription {
                repeat: 1,
                width: 1,
                typ: ColumnDataType::Float,
            }
        );
    }

    #[test]
    fn test_parse_many_repeats() {
        let s = "100E";
        assert_eq!(
            s.parse::<ColumnDataDescription>().unwrap(),
            ColumnDataDescription {
                repeat: 100,
                width: 1,
                typ: ColumnDataType::Float,
            }
        );
    }

    #[test]
    fn test_parse_with_width() {
        let s = "1E26";
        assert_eq!(
            s.parse::<ColumnDataDescription>().unwrap(),
            ColumnDataDescription {
                repeat: 1,
                width: 26,
                typ: ColumnDataType::Float,
            }
        );
    }

    #[test]
    fn test_creating_data_description() {
        let concrete_desc = ColumnDescription::new("FOO")
            .with_type(ColumnDataType::Int)
            .that_repeats(10)
            .create()
            .unwrap();
        assert_eq!(concrete_desc.name, "FOO".to_string());
        assert_eq!(concrete_desc.data_type.repeat, 10);
        assert_eq!(concrete_desc.data_type.width, 1);

        /* Do not call `with_type` */
        let bad_desc = ColumnDescription::new("FOO").create();
        assert!(bad_desc.is_err());
    }
}
