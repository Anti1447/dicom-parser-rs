use crate::byte_parser::{le_u16, le_u32};
use crate::tag::Tag;

fn length_is_u32(bytes: &[u8]) -> bool {
    (bytes[0] == b'O' && bytes[1] == b'B') ||
        (bytes[0] == b'O' && bytes[1] == b'W') ||
        (bytes[0] == b'S' && bytes[1] == b'Q') ||
        (bytes[0] == b'O' && bytes[1] == b'F') ||
        (bytes[0] == b'U' && bytes[1] == b'T') ||
        (bytes[0] == b'U' && bytes[1] == b'N')
}

#[derive(Debug, Clone, Copy)]
pub struct Attribute {
    pub tag: Tag,
    pub vr: [u8;2],
    pub length: usize,
    pub data_position: usize
}

impl Attribute {
    pub fn ele(bytes: &[u8]) -> Attribute {
        let mut attr = Attribute{
            tag: Tag::from_bytes(&bytes),
            vr: [bytes[4], bytes[5]],
            length: 0,
            data_position: 0
        };

        if length_is_u32(&bytes[4..]) {
            attr.length = le_u32(&bytes[8..]) as usize;
            attr.data_position = 12;
        } else {
            attr.length = le_u16(&bytes[6..]) as usize;
            attr.data_position = 8;
        }

        attr 
    }

    pub fn ile(bytes: &[u8]) -> Attribute {
        Attribute{
            tag: Tag::from_bytes(&bytes),
            vr: [b'U', b'N'],
            length: le_u32(&bytes[4..]) as usize,
            data_position: 8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Attribute;
    use crate::tag::Tag;

    #[test]
    fn ele_16_len() {
        let bytes = vec![8,0, 8,0, 0x43,0x53, 0x16, 00];
        let attr = Attribute::ele(&bytes);
        assert_eq!(attr.tag, Tag::new(8, 8));
        assert_eq!(attr.vr[0], b'C');
        assert_eq!(attr.vr[1], b'S');
        assert_eq!(attr.length, 22);
    }

    #[test]
    fn ele_32_len() {
        let bytes = vec![2,0, 1,0, 0x4F,0x42, 0,0, 2,0,0,0];
        let attr = Attribute::ele(&bytes);
        assert_eq!(attr.tag, Tag::new(2, 1));
        assert_eq!(attr.vr[0], b'O');
        assert_eq!(attr.vr[1], b'B');
        assert_eq!(attr.length, 2);
    }

    #[test]
    fn ile() {
        let bytes = vec![8,0, 8,0, 0x16,0,0,0];
        let attr = Attribute::ile(&bytes);
        assert_eq!(attr.tag, Tag::new(8, 8));
        assert_eq!(attr.length, 22);
    }
}