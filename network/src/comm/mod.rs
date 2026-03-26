pub mod events;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

pub async fn write_msg<M>(writer: &mut OwnedWriteHalf, msg: M) -> std::io::Result<()>
where
    M: Serialize,
{
    let bytes = bincode::serialize(&msg).unwrap();
    let len = bytes.len() as u32;

    writer.write_u32(len).await?;
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_msg<M>(reader: &mut OwnedReadHalf) -> std::io::Result<M>
where
    M: for<'a> Deserialize<'a>,
{
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len == 0 {
        return Err(std::io::Error::other("todo"));
    }

    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await?;

    bincode::deserialize(&data).map_err(|_| std::io::Error::other("todo"))
}
