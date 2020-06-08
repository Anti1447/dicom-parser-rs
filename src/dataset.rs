use crate::attribute::Attribute;

#[derive(PartialEq)]
pub enum Control {
    Element, // skip data
    Data,    // send data
    Stop,    // stop parsing
}

pub trait Callback {
    fn element(&mut self, attribute: &Attribute) -> Control;
    fn data(&mut self, attribute: &Attribute, data: &[u8]);
}

pub struct Parser<T: Callback> {
    pub callback: T,
    buffer: Vec<u8>,
    buffer_position: usize, // read position in current buffer
    data_position: usize,   // position from first byte parsed
    element_data_bytes_remaining: usize,
    state: Control,
    attribute: Option<Attribute>
}

impl<T: Callback> Parser<T> {
    pub fn new(callback: T) -> Parser<T> {
        Parser {
            callback,
            buffer: vec![],
            buffer_position: 0,
            data_position: 0,
            attribute: None,
            element_data_bytes_remaining: 0,
            state: Control::Element,
        }
    }

    pub fn parse(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(&bytes);

        while self.state != Control::Stop {
            if self.element_data_bytes_remaining > 0 {
                if (self.buffer.len() - self.buffer_position) >= self.element_data_bytes_remaining {
                    if self.state == Control::Data {
                        self.callback.data(
                            &self.attribute.unwrap(),
                            &self.buffer[self.buffer_position
                                ..self.buffer_position + self.element_data_bytes_remaining],
                        );
                        self.state = Control::Element;
                    }

                    self.buffer_position += self.element_data_bytes_remaining;
                    self.data_position += self.element_data_bytes_remaining;
                    self.element_data_bytes_remaining = 0;
                } else {
                    return;
                }
            }

            if (self.buffer.len() - self.buffer_position) >= 10 {
                let mut attribute = Attribute::ele(&self.buffer[self.buffer_position..]);
                self.buffer_position += attribute.data_position;
                self.data_position += attribute.data_position;
                attribute.data_position = self.data_position;
                self.element_data_bytes_remaining = attribute.length;
                self.state = self.callback.element(&attribute);
                self.attribute = Some(attribute);
            } else {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Callback, Control, Parser};
    use crate::attribute::Attribute;
    use crate::tag::Tag;
    use crate::vr::VR;

    struct TestCallback {
        pub attributes: Vec<Attribute>,
        pub data: Vec<Vec<u8>>,
    }

    impl Callback for TestCallback {
        fn element(&mut self, attribute: &Attribute) -> Control {
            //println!("{:?}", attribute);
            self.attributes.push(*attribute);
            Control::Data
        }

        fn data(&mut self, _attribute: &Attribute, data: &[u8]) {
            //println!("data of len {:?}", data.len());
            self.data.push(data.to_vec());
        }
    }

    fn make_dataset() -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&vec![0x02, 0x00, 0x00, 0x00, b'U', b'L', 4, 0, 0, 0, 0, 0]);
        bytes.extend_from_slice(&vec![
            0x02, 0x00, 0x01, 0x00, b'O', b'B', 0, 0, 2, 0, 0, 0, 0, 1,
        ]);

        bytes
    }


    #[test]
    fn full_parse() {
        let callback = TestCallback {
            attributes: vec![],
            data: vec![],
        };
        let mut parser = Parser::<TestCallback>::new(callback);
        let bytes = make_dataset();
        parser.parse(&bytes);
        assert_eq!(parser.callback.attributes.len(), 2);
        assert_eq!(parser.callback.attributes[0].tag, Tag::new(2, 0));
        assert_eq!(parser.callback.attributes[0].vr, Some(VR::UL));
        assert_eq!(parser.callback.attributes[0].length, 4);
        assert_eq!(parser.callback.attributes[1].tag, Tag::new(2, 1));
        assert_eq!(parser.callback.attributes[1].vr, Some(VR::OB));
        assert_eq!(parser.callback.attributes[1].length, 2);
        assert_eq!(parser.callback.data.len(), 2);
        assert_eq!(parser.callback.data[0].len(), 4);
        assert_eq!(parser.callback.data[0][0], 0);
        assert_eq!(parser.callback.data[0][1], 0);
        assert_eq!(parser.callback.data[0][2], 0);
        assert_eq!(parser.callback.data[0][3], 0);
        assert_eq!(parser.callback.data[1].len(), 2);
        assert_eq!(parser.callback.data[1][0], 0);
        assert_eq!(parser.callback.data[1][1], 1);
    }

    #[test]
    fn streaming_parse() {
        let callback = TestCallback {
            attributes: vec![],
            data: vec![],
        };
        let mut parser = Parser::<TestCallback>::new(callback);
        let bytes = make_dataset();
        parser.parse(&bytes[0..5]);
        parser.parse(&bytes[5..9]);
        parser.parse(&bytes[9..19]);
        parser.parse(&bytes[19..]);
        assert_eq!(parser.callback.attributes.len(), 2);
        assert_eq!(parser.callback.attributes[0].tag, Tag::new(2, 0));
        assert_eq!(parser.callback.attributes[0].vr, Some(VR::UL));
        assert_eq!(parser.callback.attributes[0].length, 4);
        assert_eq!(parser.callback.attributes[1].tag, Tag::new(2, 1));
        assert_eq!(parser.callback.attributes[1].vr, Some(VR::OB));
        assert_eq!(parser.callback.attributes[1].length, 2);
        assert_eq!(parser.callback.data.len(), 2);
        assert_eq!(parser.callback.data[0].len(), 4);
        assert_eq!(parser.callback.data[0][0], 0);
        assert_eq!(parser.callback.data[0][1], 0);
        assert_eq!(parser.callback.data[0][2], 0);
        assert_eq!(parser.callback.data[0][3], 0);
        assert_eq!(parser.callback.data[1].len(), 2);
        assert_eq!(parser.callback.data[1][0], 0);
        assert_eq!(parser.callback.data[1][1], 1);
    }

    struct StopCallback {
        pub element_count: usize,
        pub data_count: usize

    }

    impl Callback for StopCallback {
        fn element(&mut self, _attribute: &Attribute) -> Control {
            self.element_count += 1;
            Control::Stop
        }

        fn data(&mut self, _attribute: &Attribute, _data: &[u8]) {
            self.data_count += 1;
        }
    }


    #[test]
    fn parse_stops() {
        let callback = StopCallback {element_count: 0, data_count: 0};
        let mut parser = Parser::<StopCallback>::new(callback);
        let bytes = make_dataset();
        parser.parse(&bytes);
        assert_eq!(parser.callback.element_count, 1);
        assert_eq!(parser.callback.data_count, 0);
    }

}
