use crate::{Result, ServerToClientMessage, StorageScope};
use pinhole_protocol::{network::send_response, storage::StateMap};

pub struct Context<'a> {
    pub state_map: StateMap,

    pub(crate) stream: &'a mut async_std::net::TcpStream,
}

impl Context<'_> {
    pub async fn store(
        &mut self,
        scope: StorageScope,
        key: impl ToString,
        value: impl ToString,
    ) -> Result<()> {
        let key = key.to_string();
        let value = value.to_string();
        send_response(
            self.stream,
            ServerToClientMessage::Store { scope, key, value },
        )
        .await
    }

    pub async fn redirect(&mut self, path: impl ToString) -> Result<()> {
        let path = path.to_string();
        send_response(self.stream, ServerToClientMessage::RedirectTo { path }).await
    }
}
