use crate::encoding::Encoding;
use crate::handler::Handler;
use crate::parser::attribute::AttributeParser;
use crate::parser::ParseError;
use crate::parser::ParseResult;
use crate::parser::ParseState;
use crate::parser::Parser;
use std::marker::PhantomData;

pub struct DataSetParser<T: Encoding> {
    parser: Option<Box<dyn Parser<T>>>,
    total_bytes_consumed: usize,
}

impl<T: 'static + Encoding> DataSetParser<T> {
    pub fn default() -> DataSetParser<T> {
        let parser = Box::new(AttributeParser::<T> {
            phantom: PhantomData,
        });

        DataSetParser {
            parser: Some(parser),
            total_bytes_consumed: 0,
        }
    }
}

impl<T: 'static + Encoding> Parser<T> for DataSetParser<T> {
    fn parse(&mut self, handler: &mut dyn Handler, bytes: &[u8]) -> Result<ParseResult<T>, ()> {
        let mut remaining_bytes = bytes;
        let mut bytes_consumed = 0;

        while !remaining_bytes.is_empty() {
            match &mut self.parser {
                Some(parser) => {
                    let result = parser.parse(handler, remaining_bytes)?;
                    bytes_consumed += result.bytes_consumed;
                    self.total_bytes_consumed += result.bytes_consumed;
                    remaining_bytes = &remaining_bytes[result.bytes_consumed..];
                    match result.state {
                        ParseState::Cancelled => {
                            self.parser = None;
                            return Ok(ParseResult::cancelled(bytes_consumed));
                        }
                        ParseState::Incomplete => {
                            return Ok(ParseResult::incomplete(bytes_consumed));
                        }
                        ParseState::Partial => {
                            self.parser = result.parser;
                            continue;
                        }
                        ParseState::Completed => {
                            self.parser = Some(Box::new(AttributeParser::<T> {
                                phantom: PhantomData,
                            }));
                            continue;
                        }
                    }
                }
                None => return Err(()), // parsing cannot continue - either cancelled or completed
            };
        }
        Ok(ParseResult::completed(bytes_consumed))

        /*        if last_parse_state == ParseState::Completed {
            Ok(ParseResult::completed(bytes_consumed))
        } else {
            Ok(ParseResult::completed(bytes_consumed))
        }*/
    }
}

// returns
//  number of bytes parsed
//  true if cancelled, false, if not
pub fn parse_full<T: 'static + Encoding>(
    handler: &mut dyn Handler,
    bytes: &[u8],
) -> Result<(usize, bool), ParseError> {
    let mut parser = DataSetParser::<T>::default();
    match parser.parse(handler, bytes) {
        Ok(parse_result) => match parse_result.state {
            ParseState::Cancelled => Ok((parse_result.bytes_consumed, true)),
            ParseState::Incomplete => Err(ParseError {}),
            ParseState::Partial => Err(ParseError {}),
            ParseState::Completed => Ok((parse_result.bytes_consumed, false)),
        },
        Err(()) => Err(ParseError {}),
    }
}

#[cfg(test)]
mod tests {

    use super::DataSetParser;
    use crate::parser::ParseState;
    use crate::parser::Parser;
    //use crate::parser::ParseResult;
    use crate::test::tests::read_data_set_bytes_from_file;
    //use crate::parser::attribute::AttributeParser;
    use crate::encoding::ExplicitLittleEndian;
    use crate::handler::data_set::DataSetHandler;
    //use std::marker::PhantomData;

    fn parse_ele_data_set(bytes: &[u8]) -> Result<(ParseState, usize), ()> {
        let mut handler = DataSetHandler::default();
        let mut parser = DataSetParser::<ExplicitLittleEndian>::default();
        let result = parser.parse(&mut handler, bytes).unwrap();
        Ok((result.state, result.bytes_consumed))
    }
    #[test]
    fn parse_full_ok() {
        let bytes =
            read_data_set_bytes_from_file("tests/fixtures/CT1_UNC.explicit_little_endian.dcm");
        let result = parse_ele_data_set(&bytes[..]);
        assert!(result.is_ok());
    }

    fn split_parse(bytes: &[u8], split_position: usize) -> Result<(), ()> {
        println!("split_parse @ {}", split_position);
        let mut handler = DataSetHandler::default();
        handler.print = true;
        let mut parser = DataSetParser::<ExplicitLittleEndian>::default();
        let result = parser.parse(&mut handler, &bytes[0..split_position])?;
        println!("bytes_consumed: {:?}", result.bytes_consumed);
        //println!("state: {:?}", result.state);
        let result2 = parser.parse(&mut handler, &bytes[result.bytes_consumed..])?;
        println!("bytes_consumed: {:?}", result2.bytes_consumed);
        //println!("state: {:?}", result2.state);
        assert_eq!(result2.bytes_consumed, bytes.len() - result.bytes_consumed);
        Ok(())
    }

    #[test]
    fn parse_partial_debug() {
        //let bytes = read_data_set_bytes_from_file("tests/fixtures/CT1_UNC.explicit_little_endian.dcm");
        let bytes =
            read_data_set_bytes_from_file("tests/fixtures/CT0012.fragmented_no_bot_jpeg_ls.80.dcm"); // meta ends at 352

        // 1569 + 352 = 1921 x781 (in tag of seq item )
        let result = split_parse(&bytes, 1569);
        assert!(result.is_ok());
        //println!("{:?}", result);
    }
    /*
    #[test]
    fn parse_partial_ok() {
        let bytes = read_data_set_bytes_from_file("tests/fixtures/CT0012.fragmented_no_bot_jpeg_ls.80.dcm");
        //let bytes = read_data_set_bytes_from_file("tests/fixtures/CT1_UNC.explicit_little_endian.dcm");
        for i in 0..bytes.len() {
            let result = split_parse(&bytes, i);
            assert!(result.is_ok());
        }
        //println!("{:?}", result);
    }
    */
}
