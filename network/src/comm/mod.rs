pub mod events;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

const MAX_MSG_SIZE: u32 = 10 * 1024 * 1024; // 10 MiB

pub trait PeerWriter<M> {
    fn write_to_peer(
        &mut self,
        msg: M,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
}

pub trait PeerReader<M> {
    fn read_from_peer(&mut self) -> impl std::future::Future<Output = std::io::Result<M>> + Send;
}

impl<M> PeerReader<M> for OwnedReadHalf
where
    M: for<'a> Deserialize<'a>,
{
    async fn read_from_peer(&mut self) -> std::io::Result<M> {
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf);
        if len > MAX_MSG_SIZE {
            return Err(std::io::Error::other("read_msg: msg too big"));
        }

        if len == 0 {
            return Err(std::io::Error::other("read_msg: empty msg"));
        }

        let mut data = vec![0u8; len as usize];
        self.read_exact(&mut data).await?;

        bincode::deserialize(&data)
            .map_err(|_| std::io::Error::other("read_msg: deserialization error"))
    }
}

impl<M> PeerWriter<M> for OwnedWriteHalf
where
    M: Serialize + Send,
{
    async fn write_to_peer(&mut self, msg: M) -> std::io::Result<()> {
        let bytes = bincode::serialize(&msg).unwrap();
        let len = bytes.len() as u32;

        if len > MAX_MSG_SIZE {
            return Err(std::io::Error::other("write_msg: msg too big"));
        }

        self.write_u32(len).await?;
        self.write_all(&bytes).await?;
        self.flush().await?;
        Ok(())
    }
}

/*
pub async fn write_msg<M>(writer: &mut OwnedWriteHalf, msg: M) -> std::io::Result<()>
where
    M: Serialize,
{
    let bytes = bincode::serialize(&msg).unwrap();
    let len = bytes.len() as u32;

    if len > MAX_MSG_SIZE {
        return Err(std::io::Error::other("write_msg: msg too big"));
    }

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
    let len = u32::from_be_bytes(len_buf);
    if len > MAX_MSG_SIZE {
        return Err(std::io::Error::other("read_msg: msg too big"));
    }

    if len == 0 {
        return Err(std::io::Error::other("read_msg: empty msg"));
    }

    let mut data = vec![0u8; len as usize];
    reader.read_exact(&mut data).await?;

    bincode::deserialize(&data)
        .map_err(|_| std::io::Error::other("read_msg: deserialization error"))
}
*/
