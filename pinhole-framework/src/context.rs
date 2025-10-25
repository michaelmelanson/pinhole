use crate::{MessageStream, Result, ServerToClientMessage, StorageScope};
use pinhole_protocol::{
    messages::ErrorCode,
    network::send_message_to_client,
    storage::{StateMap, StateValue},
    CapabilitySet,
};

pub struct Context<'a> {
    pub storage: StateMap,

    pub(crate) stream: &'a mut dyn MessageStream,
    pub(crate) capabilities: CapabilitySet,
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
        .map_err(|e| e.into())
    }

    pub async fn redirect(&mut self, path: impl ToString) -> Result<()> {
        let path = path.to_string();
        send_message_to_client(self.stream, ServerToClientMessage::RedirectTo { path })
            .await
            .map_err(|e| e.into())
    }

    /// Assert that a capability is supported, terminating the connection if not
    pub async fn assert_capability(&mut self, capability: &str) -> Result<()> {
        if !self.capabilities.contains(capability) {
            send_message_to_client(
                self.stream,
                ServerToClientMessage::Error {
                    code: ErrorCode::UpgradeRequired,
                    message: format!("Missing required capability: {}", capability),
                },
            )
            .await?;
            Err(format!("Missing required capability: {}", capability).into())
        } else {
            Ok(())
        }
    }
}
