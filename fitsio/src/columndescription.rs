use std::str::FromStr;

/// Description for new columns
#[derive(Debug, Clone)]
pub struct ColumnDescription {
    pub name: String,

    /// Type of the data, see the cfitsio documentation
    pub data_type: Option<ColumnDataDescription>,
}

#[derive(Debug, Clone)]
pub struct ConcreteColumnDescription {
    pub name: String,
    pub data_type: ColumnDataDescription,
}

impl ColumnDescription {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ColumnDescription {
            name: name.into(),
            data_type: None,
        }
    }

    pub fn with_type(&mut self, typ: ColumnDataType) -> &mut ColumnDescription {
        self.data_type = Some(ColumnDataDescription::scalar(typ));
        self
    }

    pub fn that_repeats(&mut self, repeat: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.repeat = repeat;
        }
        self
    }

    pub fn with_width(&mut self, width: usize) -> &mut ColumnDescription {
        if let Some(ref mut desc) = self.data_type {
            desc.width = width;
        }
        self
    }

    pub fn create(&self) -> Result<ConcreteColumnDescription, Box<::std::error::Error>> {
        match self.data_type {
            Some(ref d) => {
                Ok(ConcreteColumnDescription {
                       name: self.name.clone(),
                       data_type: d.clone(),
                   })
            }
            None => {
                Err("No data type given. Ensure the `with_type` method has been called.".into())
            }
        }
    }
}

/// Description of the column data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDataDescription {
    pub repeat: usize,
    pub width: usize,
    pub typ: ColumnDataType,
}

impl ColumnDataDescription {
    pub fn new(typ: ColumnDataType, repeat: usize, width: usize) -> Self {
        ColumnDataDescription {
            repeat: repeat,
            width: width,
            typ: typ,
        }
    }

    /// Shortcut for creating a scalar column
    pub fn scalar(typ: ColumnDataType) -> Self {
        ColumnDataDescription::new(typ, 1, 1)
    }

    pub fn vector(typ: ColumnDataType, repeat: usize) -> Self {
        ColumnDataDescription::new(typ, repeat, 1)
    }

    /* XXX These two methods force a call to clone which is wasteful of memory. I do not know if
     * this means that memory is leaked, or that destructors are needlessly called (I suspect the
     * latter) but it is fairly wasteful. On the other hand, it's unlikely this sort of thing will
     * be called in performance-critical code, and is more likely a one-dime definition. I will
     * leave it for now - SRW 2017-03-07
     * */
    pub fn repeats(&mut self, repeat: usize) -> Self {
        // TODO check that repeat >= 1
        self.repeat = repeat;
        self.clone()
    }

    pub fn width(&mut self, width: usize) -> Self {
        // TODO check that width >= 1
        self.width = width;
        self.clone()
    }
}

impl From<ColumnDataDescription> for String {
    fn from(orig: ColumnDataDescription) -> String {
        match orig.typ {
            ColumnDataType::Text => {
                if orig.width > 1 {
                    format!("{repeat}{data_type}{width}",
                            data_type = String::from(orig.typ),
                            repeat = orig.repeat,
                            width = orig.width)
                } else {
                    format!("{repeat}{data_type}",
                            data_type = String::from(orig.typ),
                            repeat = orig.repeat)
                }
            }
            _ => {
                format!("{repeat}{data_type}",
                        data_type = String::from(orig.typ),
                        repeat = orig.repeat)
            }
        }
    }
}

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
                Text => "A",
                Double => "D",
                Short => "I",
                Long => "K",
                String => "A",
            }
            .to_string()
    }
}

impl FromStr for ColumnDataDescription {
    type Err = Box<::std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            /* TODO: in nightly the following line works
            let repeat_str: String = repeat_str.into_iter().collect(); */
            let repeat_str: String = repeat_str.into_iter().cloned().collect();
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

        /* TODO: validate that the whole string has been used up */

        let width = if width_str.is_empty() {
            1
        } else {
            /* TODO: in nightly the following line works
            let width_str: String = width_str.into_iter().collect(); */
            let width_str: String = width_str.into_iter().cloned().collect();
            width_str.parse::<usize>()?
        };

        let data_type = match data_type_char {
            'E' => ColumnDataType::Float,
            'J' => ColumnDataType::Int,
            'D' => ColumnDataType::Double,
            'I' => ColumnDataType::Short,
            'K' => ColumnDataType::Long,
            'A' => ColumnDataType::String,
            _ => {
                panic!("Have not implemented str -> ColumnDataType for {}",
                       data_type_char)
            }
        };

        Ok(ColumnDataDescription {
               repeat: repeat,
               typ: data_type,
               width: width,
           })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_column_data_descriptions_builder_pattern() {
        let desc = ColumnDataDescription::scalar(ColumnDataType::Int)
            .width(100)
            .repeats(5);
        assert_eq!(desc.repeat, 5);
        assert_eq!(desc.width, 100);
    }

    #[test]
    fn from_impls() {
        {
            let desc = ColumnDataDescription::scalar(ColumnDataType::Int).repeats(5);
            assert_eq!(String::from(desc), "5J");
        }

        {
            let desc = ColumnDataDescription::scalar(ColumnDataType::Float);
            assert_eq!(String::from(desc), "1E");
        }

        {
            let desc = ColumnDataDescription::scalar(ColumnDataType::Text).width(100);
            assert_eq!(String::from(desc), "1A100");
        }
    }

    #[test]
    fn parsing() {
        let s = "1E";
        assert_eq!(s.parse::<ColumnDataDescription>().unwrap(),
                   ColumnDataDescription {
                       repeat: 1,
                       width: 1,
                       typ: ColumnDataType::Float,
                   });
    }

    #[test]
    fn parse_many_repeats() {
        let s = "100E";
        assert_eq!(s.parse::<ColumnDataDescription>().unwrap(),
                   ColumnDataDescription {
                       repeat: 100,
                       width: 1,
                       typ: ColumnDataType::Float,
                   });
    }

    #[test]
    fn parse_with_width() {
        let s = "1E26";
        assert_eq!(s.parse::<ColumnDataDescription>().unwrap(),
                   ColumnDataDescription {
                       repeat: 1,
                       width: 26,
                       typ: ColumnDataType::Float,
                   });
    }

    #[test]
    fn creating_data_description() {
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
