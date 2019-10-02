// This is based on the Serde docs JSON example with its license below.

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use crate::error::{Error, Result};
use crate::parse::{parse_type, TNetStringType};
use serde::{
    de::{
        self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
        Visitor,
    },
    forward_to_deserialize_any, Deserialize,
};
use std::ops::{AddAssign, MulAssign, Neg, SubAssign};
use std::str;

pub struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::UnusedParseData)
    }
}

impl<'de> Deserializer<'de> {
    fn parse_bool(&mut self) -> Result<bool> {
        if self.input.starts_with("4:true!") {
            self.input = &self.input["4:true!".len()..];
            Ok(true)
        } else if self.input.starts_with("5:false!") {
            self.input = &self.input["5:false!".len()..];
            Ok(false)
        } else {
            Err(Error::ParsingBool)
        }
    }

    fn last_char(&self) -> Result<char> {
        self.input.chars().last().ok_or(Error::Eof)
    }

    fn parse_unsigned<T>(&mut self) -> Result<T>
    where
        T: AddAssign<T> + MulAssign<T> + From<u8>,
    {
        let start_pos = match self.input.find(':') {
            Some(len) => len + 1,
            _ => return Err(Error::ParsingUnsigned),
        };

        let val_len: usize = match self.input[..start_pos - 1].parse() {
            Ok(v) => v,
            _ => return Err(Error::ParsingUnsigned),
        };

        let data = &self.input[start_pos..start_pos + val_len];

        let mut int = T::from(0);
        for c in data.chars() {
            int *= T::from(10);
            int += T::from(c as u8 - b'0')
        }
        self.input = &self.input[start_pos + val_len + 1..];
        Ok(int)
    }

    fn parse_signed<T>(&mut self) -> Result<T>
    where
        T: Neg<Output = T> + AddAssign<T> + SubAssign<T> + MulAssign<T> + From<i8>,
    {
        let start_pos = match self.input.find(':') {
            Some(len) => len + 1,
            _ => return Err(Error::ParsingString),
        };

        let val_len: usize = match self.input[..start_pos - 1].parse() {
            Ok(v) => v,
            _ => return Err(Error::ParsingUnsigned),
        };

        let data = &self.input[start_pos..start_pos + val_len];

        let is_negated = match data.chars().nth(0) {
            Some('-') => true,
            _ => false,
        };

        let mut num = T::from(0);
        let skip = if is_negated { 1 } else { 0 };

        for c in data.chars().skip(skip) {
            num *= T::from(10);
            let adder = T::from(c as i8 - b'0' as i8);
            if is_negated {
                num -= adder;
            } else {
                num += adder;
            }
        }

        self.input = &self.input[start_pos + val_len + 1..];
        Ok(num)
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        if let Ok(TNetStringType::Str) = parse_type(self.input.as_bytes()) {
            let start_pos = match self.input.find(':') {
                Some(len) => len + 1,
                _ => {
                    return Err(Error::ParsingString);
                }
            };
            let val_len: usize = match self.input[..start_pos - 1].parse() {
                Ok(v) => v,
                _ => return Err(Error::ParsingUnsigned),
            };
            let end_pos = val_len + start_pos;
            let val = &self.input[start_pos..end_pos];
            self.input = &self.input[end_pos + 1..];
            Ok(val)
        } else {
            Err(Error::ParsingString)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.last_char()? {
            '~' => self.deserialize_unit(visitor),
            '!' => self.deserialize_bool(visitor),
            ',' => self.deserialize_str(visitor),
            '^' => self.deserialize_f64(visitor),
            '#' => self.deserialize_i64(visitor),
            ']' => self.deserialize_seq(visitor),
            '}' => self.deserialize_map(visitor),
            _ => Err(Error::UnknownSegmentType),
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 u8 u16 char unit_struct tuple struct bytes byte_buf
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_signed()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_unsigned()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_unsigned()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("0:~") {
            self.input = &self.input["0:~".len()..];
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.starts_with("0:~") {
            self.input = &self.input["0:~".len()..];
            visitor.visit_unit()
        } else {
            Err(Error::ParsingUnit)
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Ok(TNetStringType::List) = parse_type(self.input.as_bytes()) {
            let start_pos = match self.input.find(':') {
                Some(len) => len + 1,
                _ => return Err(Error::ParsingString),
            };
            self.input = &self.input[start_pos..self.input.len() - 1];
            let value = visitor.visit_seq(TNetStringAccess::new(&mut self))?;
            Ok(value)
        } else {
            Err(Error::ParsingSeq)
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Ok(TNetStringType::Dict) = parse_type(self.input.as_bytes()) {
            let start_pos = match self.input.find(':') {
                Some(len) => len + 1,
                _ => return Err(Error::ParsingString),
            };
            self.input = &self.input[start_pos..self.input.len() - 1];
            let value = visitor.visit_map(TNetStringAccess::new(&mut self))?;
            Ok(value)
        } else {
            Err(Error::ParsingMap)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let Ok(TNetStringType::Str) = parse_type(self.input.as_bytes()) {
            let start_pos = match self.input.find(':') {
                Some(len) => len + 1,
                _ => return Err(Error::ParsingLength),
            };

            let val = &self.input[start_pos..self.input.len() - 1];

            self.input = &self.input[self.input.len()..];

            visitor.visit_enum(val.into_deserializer())
        } else if let Ok(TNetStringType::Dict) = parse_type(self.input.as_bytes()) {
            let start_pos = match self.input.find(':') {
                Some(len) => len + 1,
                _ => return Err(Error::ParsingUnsigned),
            };

            self.input = &self.input[start_pos..self.input.len() - 1];

            let value = visitor.visit_enum(Enum::new(self))?;
            Ok(value)
        } else {
            Err(Error::ParsingEnum)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct TNetStringAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> TNetStringAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        TNetStringAccess { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for TNetStringAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.de.input.is_empty() {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, 'a> MapAccess<'de> for TNetStringAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.input.is_empty() {
            return Ok(None);
        }
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    // handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ParsingUnitVariant)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::from_str;
    use super::{Error, Result};
    use crate::error::Error::Message;
    use serde::Deserialize;
    use std::f32;
    use std::f64;

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
        }

        let j = "27:3:int,1:1#3:seq,8:1:a,1:b,]}";
        let expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
        };
        assert_eq!(Ok(expected), from_str(j));
    }

    #[test]
    fn test_struct_with_neg() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(i32);

        let t = "2:-1#";
        let expected = Test(-1);
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_option() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            option: Option<i32>,
        }

        let input = "13:6:option,1:1#}";
        let expected = Test { option: Some(1) };
        assert_eq!(Ok(expected), from_str(input));

        let input = "12:6:option,0:~}";
        let expected = Test { option: None };
        assert_eq!(Ok(expected), from_str(input));
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Foo,
            Newtype(u32),
            N(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let j = "4:Unit,";
        let expected = E::Unit;
        assert_eq!(Ok(expected), from_str(j));

        let j = "3:Foo,";
        let expected = E::Foo;
        assert_eq!(Ok(expected), from_str(j));

        let j = "14:7:Newtype,1:1#}";
        let expected = E::Newtype(1);
        assert_eq!(Ok(expected), from_str(j));

        let j = "9:1:N,2:20#}";
        let expected = E::N(20);
        assert_eq!(Ok(expected), from_str(j));

        let j = "19:5:Tuple,8:1:1#1:2#]}";
        let expected = E::Tuple(1, 2);
        assert_eq!(Ok(expected), from_str(j));

        let j = "20:6:Struct,8:1:a,1:1#}}";
        let expected = E::Struct { a: 1 };
        assert_eq!(Ok(expected), from_str(j));
    }

    #[test]
    fn test_unit() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(());

        let t = "0:~";
        let expected = Test(());
        assert_eq!(Ok(expected), from_str(t))
    }

    #[test]
    fn test_bool() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(bool);

        let t = "4:true!";
        let expected = Test(true);
        assert_eq!(Ok(expected), from_str(t));

        let t = "5:false!";
        let expected = Test(false);
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_char() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(char);

        let t = "1:a,";
        let expected = Test('a');
        assert_eq!(Ok(expected), from_str(t));

        let t = "2:ab,";
        let actual: Result<Test> = from_str(t);
        assert_eq!(
            Err(Message(
                "invalid value: string \"ab\", expected a character".into()
            )),
            actual
        );
    }

    #[test]
    fn test_string() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(String);

        let t = "1:a,";
        let expected = Test(String::from("a"));
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_str() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test<'a>(&'a str);

        let t = "1:a,";
        let expected = Test("a");
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_u8() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(u8);

        let t = "3:255#";
        let expected = Test(u8::max_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_u16() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(u16);

        let t = "5:65535#";
        let expected = Test(u16::max_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_u32() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(u32);

        let t = "10:4294967295#";
        let expected = Test(u32::max_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_u64() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(u64);

        let t = "20:18446744073709551615#";
        let expected = Test(u64::max_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_i8() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(i8);

        let t = "3:127#";
        let expected = Test(i8::max_value());
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:-128#";
        let expected = Test(i8::min_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_i16() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(i16);

        let t = "5:32767#";
        let expected = Test(i16::max_value());
        assert_eq!(Ok(expected), from_str(t));

        let t = "6:-32768#";
        let expected = Test(i16::min_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_i32() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(i32);

        let t = "10:2147483647#";
        let expected = Test(i32::max_value());
        assert_eq!(Ok(expected), from_str(t));

        let t = "11:-2147483648#";
        let expected = Test(i32::min_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_i64() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(i64);

        let t = "19:9223372036854775807#";
        let expected = Test(i64::max_value());
        assert_eq!(Ok(expected), from_str(t));

        let t = "20:-9223372036854775808#";
        let expected = Test(i64::min_value());
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_unimplemented_f32() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(f32);

        let actual: Result<Test> = from_str("4:1.00^");
        assert_eq!(Err(Error::UnsupportedType), actual);
    }

    #[test]
    fn test_unimplemented_f64() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(f64);

        let actual: Result<Test> = from_str("4:1.00^");
        assert_eq!(Err(Error::UnsupportedType), actual);
    }

    #[test]
    #[ignore]
    fn test_f32() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(f32);

        let t = "4:1.00^";
        let expected = Test(1.00);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::MAX);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::NAN);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::INFINITY);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::NEG_INFINITY);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::MIN);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f32::MIN_POSITIVE);
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    #[ignore]
    fn test_f64() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test(f64);

        let t = "4:1.00^";
        let expected = Test(1.00);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::MAX);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::NAN);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::INFINITY);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::NEG_INFINITY);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::MIN);
        assert_eq!(Ok(expected), from_str(t));

        let t = "4:1.00^";
        let expected = Test(f64::MIN_POSITIVE);
        assert_eq!(Ok(expected), from_str(t));
    }

    #[test]
    fn test_bytes() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test<'a>(&'a [u8]);

        assert_eq!(Ok(Test(&[48, 49, 50, 51, 52, 53])), from_str("6:012345,"));
    }

}
