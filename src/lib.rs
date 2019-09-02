use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum TNetString {
    Bool(bool),
    Str(String),
    Int(i64),
    Float(f64),
    Null,
    List(Vec<TNetString>),
    Dict(HashMap<String, TNetString>),
}

#[derive(Debug, PartialEq)]
pub enum TNetStringError {
    UnknownSegmentType,
    UnableToParseInt,
    UnableToParseFloat,
    NoneZeroLengthNull,
    UnableToTake,
    FoundNonStringKey,
}

fn is_digit(data: u8) -> bool {
    (b'0' <= data) && (data <= b'9')
}

fn parse_tag(data: &[u8]) -> Result<(&[u8], usize), TNetStringError> {
    let mut temp_str = String::new();
    let mut index = 0;

    while index < data.len() && is_digit(data[index]) {
        temp_str.push(data[index] as char);
        index += 1;
    }

    let num = match temp_str.parse::<usize>() {
        Ok(n) => n,
        Err(_) => return Err(TNetStringError::UnableToParseInt),
    };
    index += 1;

    Ok((&data[index..], num))
}

fn take(data: &[u8], n: usize) -> Result<(&[u8], &[u8]), TNetStringError> {
    if n <= data.len() {
        Ok((&data[n..], &data[..n]))
    } else {
        Err(TNetStringError::UnableToTake)
    }
}

fn parse_pair(data: &[u8]) -> Result<(&[u8], (String, TNetString)), TNetStringError> {
    let (remain, parsed_key) = parse(data)?;
    let (remain, parsed_value) = parse(remain)?;

    match parsed_key {
        TNetString::Str(key) => Ok((remain, (key, parsed_value))),
        _ => Err(TNetStringError::FoundNonStringKey),
    }
}

// 4:true! --> ("!", "true")
fn split_data(data: &[u8]) -> Result<(&[u8], &[u8]), TNetStringError> {
    let (res, num) = parse_tag(data)?;
    take(res, num)
}

pub fn parse(data: &[u8]) -> Result<(&[u8], TNetString), TNetStringError> {
    let (type_tag_content, content) = split_data(data)?;

    let (remain, type_tag) = take(type_tag_content, 1)?;
    match type_tag.first() {
        Some(b'!') => {
            let is_true = content.len() == "true".len();
            Ok((remain, TNetString::Bool(is_true)))
        }
        Some(b',') => {
            let str_content = String::from_utf8_lossy(content);
            Ok((remain, TNetString::Str(str_content.to_string())))
        }
        Some(b'#') => String::from_utf8_lossy(content)
            .parse::<i64>()
            .map(|num| (remain, TNetString::Int(num)))
            .map_err(|_| TNetStringError::UnableToParseInt),
        Some(b'^') => String::from_utf8_lossy(content)
            .parse::<f64>()
            .map(|num| (remain, TNetString::Float(num)))
            .map_err(|_| TNetStringError::UnableToParseFloat),
        Some(b'~') => {
            if !content.is_empty() {
                Err(TNetStringError::NoneZeroLengthNull)
            } else {
                Ok((remain, TNetString::Null))
            }
        }
        Some(b']') => {
            let mut list_content = vec![];
            let mut remain_content = content;

            loop {
                if remain_content.is_empty() {
                    return Ok((remain_content, TNetString::List(list_content)));
                }
                match parse(remain_content) {
                    Ok((remain, data)) => {
                        list_content.push(data);
                        remain_content = remain;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        Some(b'}') => {
            let mut dict = HashMap::new();
            let mut remain_content = content;

            loop {
                if remain_content.is_empty() {
                    return Ok((remain_content, TNetString::Dict(dict)));
                }
                match parse_pair(remain_content) {
                    Ok((remain, (key, value))) => {
                        dict.insert(key, value);
                        remain_content = remain;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        _ => Err(TNetStringError::UnknownSegmentType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn it_parses_a_pair() {
        assert_eq!(
            parse_pair(b"5:hello,5:world,"),
            Ok((
                "".as_bytes(),
                ("hello".into(), TNetString::Str("world".into()))
            ))
        );
        assert_eq!(
            parse_pair(b"5:hello,"),
            Err(TNetStringError::UnableToParseInt)
        );
    }

    #[test]
    fn it_parses_dicts() {
        let expected_hashmap = TNetString::Dict(hashmap! {
            "hello".into() => TNetString::Str("world".into())
        });

        assert_eq!(
            parse(b"16:5:hello,5:world,}"),
            Ok(("".as_bytes(), expected_hashmap))
        );

        assert_eq!(
            parse(b"0:}"),
            Ok(("".as_bytes(), TNetString::Dict(HashMap::new())))
        );

        let expected_hashmap = TNetString::Dict(hashmap! {
            "hello".into() => TNetString::Dict(hashmap! {
                "hello".into() => TNetString::Dict(hashmap! {
                    "hello".into() => TNetString::Str("world".into()),
                }),
            }),
        });

        assert_eq!(
            parse(b"40:5:hello,28:5:hello,16:5:hello,5:world,}}}"),
            Ok(("".as_bytes(), expected_hashmap))
        );
    }

    #[test]
    fn it_parses_list_content() {
        assert_eq!(
            parse(b"4:true!3:bar,"),
            Ok(("3:bar,".as_bytes(), TNetString::Bool(true)))
        );

        assert_eq!(
            parse(b"12:3:foo,3:bar,]"),
            Ok((
                "".as_bytes(),
                TNetString::List(vec![
                    TNetString::Str("foo".into()),
                    TNetString::Str("bar".into())
                ])
            ))
        );
        assert_eq!(parse(b"0:1~"), Err(TNetStringError::UnknownSegmentType));
        assert_eq!(parse(b"1:a~"), Err(TNetStringError::NoneZeroLengthNull));
    }

    #[test]
    fn it_parses_bool_content() {
        assert_eq!(
            parse(b"4:true!"),
            Ok(("".as_bytes(), TNetString::Bool(true)))
        );
        assert_eq!(
            parse(b"4:xyz%!"),
            Ok(("".as_bytes(), TNetString::Bool(true)))
        );
        assert_eq!(
            parse(b"5:false!"),
            Ok(("".as_bytes(), TNetString::Bool(false)))
        );
        assert_eq!(
            parse(b"5:aaaaa!"),
            Ok(("".as_bytes(), TNetString::Bool(false)))
        );
    }

    #[test]
    fn it_parses_str_content() {
        assert_eq!(
            parse(b"4:true,"),
            Ok(("".as_bytes(), TNetString::Str("true".into())))
        );
        assert_eq!(
            parse(b"4:xyz%,"),
            Ok(("".as_bytes(), TNetString::Str("xyz%".into())))
        );
        assert_eq!(
            parse(b"5:false,"),
            Ok(("".as_bytes(), TNetString::Str("false".into())))
        );
        assert_eq!(
            parse(b"5:true!,"),
            Ok(("".as_bytes(), TNetString::Str("true!".into())))
        );
        assert_eq!(
            parse(b"5:aaaaa,"),
            Ok(("".as_bytes(), TNetString::Str("aaaaa".into())))
        );
        assert_eq!(
            parse(b"5:,,,,,,"),
            Ok(("".as_bytes(), TNetString::Str(",,,,,".into())))
        );
        assert_eq!(
            parse(b"9:123456789,"),
            Ok(("".as_bytes(), TNetString::Str("123456789".into())))
        );
        assert_eq!(
            parse(b"10:123456789A,"),
            Ok(("".as_bytes(), TNetString::Str("123456789A".into())))
        );
        assert_eq!(
            parse(b"12:3:foo,3:bar,,"),
            Ok(("".as_bytes(), TNetString::Str("3:foo,3:bar,".into())))
        );
    }

    #[test]
    fn it_parses_integers_content() {
        assert_eq!(
            parse(b"4:1000#"),
            Ok(("".as_bytes(), TNetString::Int(1000)))
        );
        assert_eq!(parse(b"5:00000#"), Ok(("".as_bytes(), TNetString::Int(0))));
        assert_eq!(parse(b"2:-1#"), Ok(("".as_bytes(), TNetString::Int(-1))));
        assert_eq!(parse(b"5:00001#"), Ok(("".as_bytes(), TNetString::Int(1))));
        assert_eq!(
            parse(b"5:12340#"),
            Ok(("".as_bytes(), TNetString::Int(12340)))
        );
        assert_eq!(parse(b"5:,,,,,#"), Err(TNetStringError::UnableToParseInt));
    }

    #[test]
    fn it_parses_float_content() {
        assert_eq!(
            parse(b"4:1.00^"),
            Ok(("".as_bytes(), TNetString::Float(1.0000)))
        );
        assert_eq!(
            parse(b"5:00000^"),
            Ok(("".as_bytes(), TNetString::Float(0.0)))
        );
        assert_eq!(
            parse(b"4:-1.0^"),
            Ok(("".as_bytes(), TNetString::Float(-1.0)))
        );
        assert_eq!(
            parse(b"5:00001^"),
            Ok(("".as_bytes(), TNetString::Float(1.0)))
        );
        assert_eq!(
            parse(b"5:123.4^"),
            Ok(("".as_bytes(), TNetString::Float(123.4)))
        );
        assert_eq!(parse(b"5:,,,,,^"), Err(TNetStringError::UnableToParseFloat));
    }

    #[test]
    fn it_parses_null_content() {
        assert_eq!(parse(b"0:~"), Ok(("".as_bytes(), TNetString::Null)));
        assert_eq!(parse(b"0:1~"), Err(TNetStringError::UnknownSegmentType));
        assert_eq!(parse(b"1:a~"), Err(TNetStringError::NoneZeroLengthNull));
    }

    #[test]
    fn it_parses_tag() {
        assert_eq!(parse_tag(b"4:true!"), Ok(("true!".as_bytes(), 4)));
        assert_eq!(parse_tag(b"10:false!"), Ok(("false!".as_bytes(), 10)));
    }

    #[test]
    fn it_takes_data() {
        assert_eq!(
            take(b"4:true!", 2),
            Ok(("true!".as_bytes(), "4:".as_bytes()))
        );
        assert_eq!(
            take(b"4:true!", 7),
            Ok(("".as_bytes(), "4:true!".as_bytes()))
        );
        assert_eq!(take(b"4:true!", 8), Err(TNetStringError::UnableToTake));
    }

    #[test]
    fn it_splits_data() {
        let (remain, parsed) = split_data(b"4:true!").unwrap();
        assert_eq!(
            ((
                String::from_utf8(remain.to_vec()).unwrap(),
                String::from_utf8(parsed.to_vec()).unwrap()
            )),
            (("!".into(), "true".into()))
        );

        assert_eq!(split_data(b"10:false!"), Err(TNetStringError::UnableToTake));
        assert_eq!(
            split_data(b"10:123456789A!"),
            Ok(("!".as_bytes(), "123456789A".as_bytes()))
        );
    }

}
