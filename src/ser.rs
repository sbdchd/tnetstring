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

use serde;
use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub struct Serializer {
    output: Vec<String>,
}

// TODO(sbdchd): add a to_bytes func
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: vec![String::new()],
    };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .last()
        .ok_or(Error::StackProblem)
        .map(String::from)
}

// due to the structure of serde serializers being broken into multiple steps we
// use a stack since we are required to know the length of a sequences and dicts
// before we can serialize them.
impl Serializer {
    fn add_to_output(&mut self, v: &str) {
        if let Some(val) = self.output.last_mut() {
            val.push_str(v);
        }
    }

    fn add_string_to_stack(&mut self) {
        self.output.push(String::new())
    }

    fn pop_string(&mut self) -> Option<String> {
        self.output.pop()
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();

    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.add_to_output(if v { "4:true!" } else { "5:false!" });
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let num = &v.to_string();
        self.add_to_output(&format!("{}:{}#", num.len(), num));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let num = &v.to_string();
        self.add_to_output(&format!("{}:{}#", num.len(), num));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let num = &v.to_string();
        self.add_to_output(&format!("{}:{}^", num.len(), num));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.add_to_output(&format!("{}:{},", v.len(), v));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.add_to_output(&format!("{}:{},", v.len(), String::from_utf8(v.to_vec())?));
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.add_to_output("0:~");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;

        if let Some(val) = self.output.last_mut() {
            *val = format!("{}:{}}}", val.len(), val);
        }
        Ok(())
    }

    // `len` is the number of elements
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.add_string_to_stack();
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        variant.serialize(&mut *self)?;
        self.add_string_to_stack();
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.add_string_to_stack();
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}]",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}]",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}]",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}]",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        if let Some(val) = self.output.last_mut() {
            *val = format!("{}:{}}}", val.len(), val);
        }
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}}}",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}}}",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.add_string_to_stack();
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        if let Some(most_recent_string) = self.pop_string() {
            self.add_to_output(&format!(
                "{}:{}}}",
                most_recent_string.len(),
                most_recent_string
            ));
        }
        if let Some(val) = self.output.last_mut() {
            *val = format!("{}:{}}}", val.len(), val);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::to_string;
    use maplit::hashmap;
    use serde::Serialize;

    #[test]
    fn test_bool() {
        let test = true;
        let expected = "4:true!";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = false;
        let expected = "5:false!";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_str() {
        let test = "true";
        let expected = "4:true,";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = "false";
        let expected = "5:false,";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = "3:foo,3:bar,";
        let expected = "12:3:foo,3:bar,,";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_ints() {
        let test = -1;
        let expected = "2:-1#";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = 12340;
        let expected = "5:12340#";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = 0;
        let expected = "1:0#";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_floats() {
        let test = 1.00;
        let expected = "1:1^";
        assert_eq!(to_string(&test).unwrap(), expected);
        let test = -1.0;
        let expected = "2:-1^";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = 1.25;
        let expected = "4:1.25^";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = 123.4;
        let expected = "5:123.4^";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_vec() {
        let test = vec!["foo", "bar"];
        let expected = "12:3:foo,3:bar,]";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = vec![10, 10];
        let expected = "10:2:10#2:10#]";
        assert_eq!(to_string(&test).unwrap(), expected);

        let test = vec![vec![10, 10]];
        let expected = "14:10:2:10#2:10#]]";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_tuple() {
        let expected = "12:3:foo,3:bar,]";
        let test = ("foo", "bar");
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Serialize)]
        struct Test(&'static str, &'static str);
        let test = Test("foo", "bar");
        let expected = "12:3:foo,3:bar,]";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_tuple_variant() {
        #[derive(Serialize)]
        enum E {
            T(&'static str, &'static str),
        }
        let test = E::T("foo", "bar");
        let expected = "20:1:T,12:3:foo,3:bar,]}";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Serialize)]
        struct Test(&'static str);

        let test = Test("foo");
        let expected = "3:foo,";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_newtype_variant() {
        #[derive(Serialize)]
        enum Test {
            T(&'static str),
        }

        let test = Test::T("foo");
        let expected = "10:1:T,3:foo,}".into();
        assert_eq!(to_string(&test), Ok(expected));
    }

    #[test]
    fn test_null() {
        let test: Option<()> = None;
        let expected = "0:~";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_map() {
        use std::collections::HashMap;
        let test: HashMap<&'static str, &'static str> = hashmap! {
            "hello" => "world"
        };
        let expected = "16:5:hello,5:world,}";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct T {
            int: u32,
        }

        let test = T { int: 10 };
        let expected = "11:3:int,2:10#}";
        assert_eq!(to_string(&test).unwrap(), expected);

        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };
        let expected = "27:3:int,1:1#3:seq,8:1:a,1:b,]}";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_struct_variant() {
        #[derive(Serialize)]
        enum Test {
            A { b: i32 },
        }

        let test = Test::A { b: 10 };
        let expected = "16:1:A,9:1:b,2:10#}}";
        assert_eq!(to_string(&test).unwrap(), expected);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let u = E::Unit;
        let expected = "4:Unit,".into();
        assert_eq!(to_string(&u), Ok(expected));

        let n = E::Newtype(1);
        let expected = "14:7:Newtype,1:1#}".into();
        assert_eq!(to_string(&n), Ok(expected));

        let t = E::Tuple(1, 2);
        let expected = "19:5:Tuple,8:1:1#1:2#]}".into();
        assert_eq!(to_string(&t), Ok(expected));

        let s = E::Struct { a: 1 };
        let expected = "20:6:Struct,8:1:a,1:1#}}".into();
        assert_eq!(to_string(&s), Ok(expected));
    }
}
