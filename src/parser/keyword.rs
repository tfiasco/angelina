use std::str::FromStr;
use std::string::ToString;

use strum_macros::EnumString;

#[derive(Display, EnumString, Debug, Clone, Eq, PartialEq)]
pub enum Keyword {
    SHOW,
    CREATE,
    DROP,
    SELECT,
    INSERT,
    UPDATE,
    DELETE,
    VERTEX,
    EDGE,
    LABEL,
    PROPERTY,
    KEY,
    PROPERTIES,
    VALUES,
    ID,
    FROM,
    WHERE,
    IS,
    NOT,
    NULL,
    AND,
    OR,
    TRUE,
    FALSE,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_keyword() {
        let keyword = Keyword::from_str("SELECT").unwrap();
        assert_eq!(Keyword::SELECT, keyword);
        assert_eq!("SELECT", keyword.to_string());

        match Keyword::from_str("xxx") {
            Ok(keyword) => panic!("should error"),
            Err(error) => assert_eq!("Matching variant not found", error.to_string()),
        }
    }
}
