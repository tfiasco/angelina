use bytes::{Buf, BufMut, BytesMut};

use crate::datamodel::constants::STRING_TERM;

pub struct Buffer {
    bytes: BytesMut
}


impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            bytes: BytesMut::new()
        }
    }

    pub fn from(arr: &[u8]) -> Buffer {
        Buffer {
            bytes: BytesMut::from(arr)
        }
    }

    pub fn get_u8(&mut self) -> u8 {
        self.bytes.get_u8()
    }

    pub fn get_u32(&mut self) -> u32 {
        self.bytes.get_u32()
    }

    pub fn get_u64(&mut self) -> u64 {
        self.bytes.get_u64()
    }

    pub fn get_string_raw(&mut self) -> Vec<u8> {
        let mut string = Vec::new();
        let mut next = self.bytes.get_u8();
        while next != STRING_TERM {
            string.push(next);
            next = self.bytes.get_u8();
        }
        string
    }

    pub fn get_string_utf8(&mut self) -> String {
        let string = self.get_string_raw();
        String::from_utf8(string).unwrap()
    }

    pub fn put_u8(&mut self, n: u8) {
        self.bytes.put_u8(n)
    }

    pub fn put_u32(&mut self, n: u32) {
        self.bytes.put_u32(n)
    }

    pub fn put_u64(&mut self, n: u64) {
        self.bytes.put_u64(n)
    }

    pub fn put_string(&mut self, data: &str) {
        self.bytes.put_slice(data.as_bytes());
        self.bytes.put_u8(STRING_TERM)
    }

    pub fn put_slice(&mut self, data: &[u8]) {
        self.bytes.put_slice(data);
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.bytes()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }

    pub fn advance(&mut self, n: usize) {
        self.bytes.advance(n)
    }

    pub fn has_remaining(&self) -> bool {
        self.bytes.has_remaining()
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let mut buf = Buffer::new();
        let i = 233;
        buf.put_u32(i);
        assert_eq!(i, buf.get_u32());

        let arr: [u8; 4] = [0, 0, 0, 0xe9];
        let mut buf = Buffer::from(&arr);
        assert_eq!(233, buf.get_u32());
    }

    #[test]
    fn test_get_put() {
        let mut buf = Buffer::new();
        for i in 0..10 {
            buf.put_u8(i);
        }
        for i in 10..1 {
            assert_eq!(i, buf.get_u8());
        }

        for i in 0..10 {
            buf.put_u32(i);
        }
        for i in 10..1 {
            assert_eq!(i, buf.get_u32());
        }

        for i in 0..10 {
            buf.put_u64(i);
        }
        for i in 10..1 {
            assert_eq!(i, buf.get_u64());
        }

        for i in 0..10 {
            buf.put_string(format!("{}", i).as_str());
        }
        for i in 10..1 {
            assert_eq!(format!("{}", i), buf.get_string_utf8());
        }
    }

    #[test]
    fn test_advance() {
        let mut buf = Buffer::from(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3]);
        assert_eq!(1, buf.get_u32());
        buf.advance(4);
        assert_eq!(3, buf.get_u32());
    }
}

