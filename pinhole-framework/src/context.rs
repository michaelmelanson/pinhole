use crate::{Result, ServerToClientMessage, StorageScope};
use pinhole_protocol::{
    network::send_message_to_client,
    storage::{StateMap, StateValue},
};

pub struct Context<'a> {
    pub storage: StateMap,

    pub(crate) stream: &'a mut async_std::net::TcpStream,
}

impl Context<'_> {
    pub async fn store(
        &mut self,
        scope: StorageScope,
        key: impl ToString,
        value: impl Into<StateValue>,
    ) -> Result<()> {
        let key = key.to_string();
        let value = value.into();
        send_message_to_client(
            self.stream,
            ServerToClientMessage::Store { scope, key, value },
        )
        .await
    }

    pub async fn redirect(&mut self, path: impl ToString) -> Result<()> {
        let path = path.to_string();
        send_message_to_client(self.stream, ServerToClientMessage::RedirectTo { path }).await
    }
}
