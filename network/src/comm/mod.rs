pub mod events;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;

use crate::comm::events::NetworkMessage;

const MAX_MSG_SIZE: u32 = 10 * 1024 * 1024; // 10 MiB

pub trait P2PSender: Send + Sync {
    fn send(
        &mut self,
        msg: NetworkMessage,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
}

pub trait P2PReceiver: Send + Sync {
    fn recieve(
        &mut self,
    ) -> impl std::future::Future<Output = std::io::Result<NetworkMessage>> + Send;
}

pub trait P2PMessenger: P2PSender + P2PReceiver {
    fn send_receive(
        &mut self,
        req: NetworkMessage,
    ) -> impl std::future::Future<Output = std::io::Result<NetworkMessage>> + Send {
        async {
            self.send(req).await?;
            self.recieve().await
        }
    }
}

impl P2PSender for OwnedWriteHalf {
    async fn send(&mut self, msg: NetworkMessage) -> std::io::Result<()> {
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

impl P2PReceiver for OwnedReadHalf {
    async fn recieve(&mut self) -> std::io::Result<NetworkMessage> {
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
