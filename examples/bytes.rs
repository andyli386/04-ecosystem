use anyhow::Result;
use bytes::{BufMut, BytesMut};
fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"Hello world\n");
    buf.put(&b"goodby world"[..]);
    buf.put_i64(0xdeadbeef);
    println!("{:?}", buf);
    let a = buf.split();
    println!("{:?}", a);

    let mut b = a.freeze();
    // let pos = b.binary_search(&10).unwrap();
    let c = b.split_to(12);
    println!("{:?}", c);
    println!("{:?}", b);

    Ok(())
}
