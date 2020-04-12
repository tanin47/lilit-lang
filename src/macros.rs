#[macro_export]
macro_rules! unwrap {
        ($enum:path, $expr:expr) => {{
            if let $enum(item) = $expr {
                item
            } else {
                panic!("Unexpected {:#?}", $expr)
            }
        }};
    }

#[macro_export]
macro_rules! unwrap2 {
        ($enum:path, $expr:expr) => {{
            if let $enum(item, item2) = $expr {
                (item, item2)
            } else {
                panic!("Unexpected {:#?}", $expr)
            }
        }};
    }
