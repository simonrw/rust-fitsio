//! Handling column descriptions
//!
//! Columns are represented as
//! [`ConcreteColumnDescription`](struct.ConcreteColumnDescription.html). This is constructed
//! through the builder pattern, by creating a [`ColumnDescription`](struct.ColumnDescription.html)
//! and calling [`create`](struct.ColumnDescription.html#method.create)
use errors::Result;
use std::str::FromStr;

/// Description for new columns
#[derive(Debug, Clone)]
pub struct ColumnDescription {
    /// Name of the column
    pub name: String,

    /// Type of the data, see the cfitsio documentation
    pub data_type: Option<ColumnDataDescription>,
}

/// Concrete representation of the description of a column
#[derive(Debug, Clone, PartialEq)]
pub struct ConcreteColumnDescription {
    /// Name of the column
    pub name: String,

    /// Type of the data, see the cfitsio documentation
    pub data_type: ColumnDataDescription,
}

impl ColumnDescription {
    /// Create a new [`ColumnDescription`](struct.ColumnDescription.html) from a name
    pub fn new<T: Into<String>>(name: T) -> Self {
        ColumnDescription {
            name: name.into(),
            data_type: None,
        }
    }

    /// Add a data type to the column description
    pub fn with_type(&mut self, typ: ColumnDataType) -> &mut ColumnDescription {
        self.data_type = Some(ColumnDataDescription::scalar(typ));
        self
    }

    /// Make the column repeat
    pub fn that_repeats(&mut self, repeat: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.repeat = repeat;
        }
        self
    }

    /// Define the column width
    pub fn with_width(&mut self, width: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.width = width;
        }
        self
    }

    /// Render the [`ColumnDescription`](struct.ColumnDescription.html) into a
    /// [`ConcreteColumnDescription`](struct.ConcreteColumnDescription.html)
    pub fn create(&self) -> Result<ConcreteColumnDescription> {
        match self.data_type {
            Some(ref d) => Ok(ConcreteColumnDescription {
                name: self.name.clone(),
                data_type: d.clone(),
            }),
            None => {
                Err("No data type given. Ensure the `with_type` method has been called.".into())
            }
        }
    }
}

/// Description of the column data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDataDescription {
    /// Does the column contain multiple values?
    pub repeat: usize,

    /// How wide is the column?
    pub width: usize,

    /// What data type does the column store?
    pub typ: ColumnDataType,
}

impl ColumnDataDescription {
    /// Create a new column data description
    pub fn new(typ: ColumnDataType, repeat: usize, width: usize) -> Self {
        ColumnDataDescription { repeat, width, typ }
    }

    /// Shortcut for creating a scalar column
    pub fn scalar(typ: ColumnDataType) -> Self {
        ColumnDataDescription::new(typ, 1, 1)
    }

    /// Shortcut for creating a vector column
    pub fn vector(typ: ColumnDataType, repeat: usize) -> Self {
        ColumnDataDescription::new(typ, repeat, 1)
    }
}

impl From<ColumnDataDescription> for String {
    fn from(orig: ColumnDataDescription) -> String {
        match orig.typ {
            ColumnDataType::Text => {
                if orig.width > 1 {
                    format!(
                        "{repeat}{data_type}{width}",
                        data_type = String::from(orig.typ),
                        repeat = orig.repeat,
                        width = orig.width
                    )
                } else {
                    format!(
                        "{repeat}{data_type}",
                        data_type = String::from(orig.typ),
                        repeat = orig.repeat
                    )
                }
            }
            _ => format!(
                "{repeat}{data_type}",
                data_type = String::from(orig.typ),
                repeat = orig.repeat
            ),
        }
    }
}

/// Types a column can represent
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDataType {
    Int,
    Float,
    Text,
    Double,
    Short,
    Long,
    String,
}

impl From<ColumnDataType> for String {
    fn from(orig: ColumnDataType) -> String {
        use self::ColumnDataType::*;

        match orig {
            Int => "J",
            Float => "E",
            Text | String => "A",
            Double => "D",
            Short => "I",
            Long => "K",
        }.to_string()
    }
}

impl FromStr for ColumnDataDescription {
    type Err = Box<::std::error::Error>;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let chars: Vec<_> = s.chars().collect();

        let mut repeat_str = Vec::new();
        let mut last_position = 0;
        for c in &chars {
            if c.is_digit(10) {
                repeat_str.push(c);
                last_position += 1;
            } else {
                break;
            }
        }

        let repeat = if repeat_str.is_empty() {
            1
        } else {
            let repeat_str: String = repeat_str.into_iter().collect();
            repeat_str.parse::<usize>()?
        };

        let data_type_char = chars[last_position];
        last_position += 1;

        let mut width_str = Vec::new();
        for c in chars.iter().skip(last_position) {
            if c.is_digit(10) {
                width_str.push(c);
            } else {
                break;
            }
        }

        let width = if width_str.is_empty() {
            1
        } else {
            let width_str: String = width_str.into_iter().collect();
            width_str.parse::<usize>()?
        };

        let data_type = match data_type_char {
            'E' => ColumnDataType::Float,
            'J' => ColumnDataType::Int,
            'D' => ColumnDataType::Double,
            'I' => ColumnDataType::Short,
            'K' => ColumnDataType::Long,
            'A' => ColumnDataType::String,
            _ => panic!(
                "Have not implemented str -> ColumnDataType for {}",
                data_type_char
            ),
        };

        Ok(ColumnDataDescription {
            repeat,
            typ: data_type,
            width,
        })
    }
}

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
