use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use std::error::Error;
use std::str;

pub async fn read_string_from_stream(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
    stream.readable().await?;

    let mut buf = [0; 4096];
    let read = stream.read(&mut buf).await?;
    let str = str::from_utf8(&buf[..read])?;

    Ok(str.to_string())
}
