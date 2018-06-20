use tokio_codec::{Decoder, Encoder};
use std::{io};
use bytes::{BufMut, BytesMut};

#[derive(Debug)]
pub struct P2p {

}

pub struct Head {
    pub ver: u16,
    pub ctrl: u8,
    pub action: u8,
    pub len: u32,
}

impl Head {
    pub fn new() -> Head {
        Head {
            ver: 0x01,
            ctrl: 0x0001,
            action: 0x0001,
            len: 0,
        }
    }
}

pub struct ChannelBuffer {
    pub head: Head,
    pub body: Vec<u8>,
}

impl ChannelBuffer {
    pub fn new() -> ChannelBuffer {
        ChannelBuffer {
            head: Head::new(),
            body: Vec::new(),
        }
    }
}

impl Encoder for P2p {
    type Item = ChannelBuffer;
    type Error = io::Error;

    fn encode(&mut self, item: ChannelBuffer, dst: &mut BytesMut) -> io::Result<()> {
        dst.put_u16_be(item.head.ver);
        dst.put_u8(item.head.ctrl);
        dst.put_u8(item.head.action);
        dst.put_u32_be(item.head.len as u32);

        dst.put_slice(item.body.as_slice());


//            for i in dst.iter() {
//                println!("Body is {}", &i);
//            }

        return Ok(());
    }
}

impl Decoder for P2p {
    type Item = ChannelBuffer;
    type Error = io::Error;

    fn decode(&mut self, _src: &mut BytesMut) -> io::Result<Option<ChannelBuffer>> {
        let req = ChannelBuffer::new();
//            req.head.ver = src.split_to(2).into_buf().get_u16_be();
//            req.head.ctrl = src.split_to(1).into_buf().get_u8();
//            req.head.action = src.split_to(1).into_buf().get_u8();
//            req.head.len = src.split_to(4).into_buf().get_u32_be();
//            req.body = src.split_to(req.head.len as usize).to_vec();

        Ok(Some(req))
    }
}