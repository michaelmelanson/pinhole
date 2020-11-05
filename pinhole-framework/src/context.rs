use crate::{Result, Scope, ServerToClientMessage};
use pinhole_protocol::{document::FormState, network::send_response};

pub struct Context<'a> {
    pub form_state: FormState,

    pub(crate) stream: &'a mut async_std::net::TcpStream,
}

impl Context<'_> {
    pub async fn store(&mut self, scope: Scope, key: String, value: String) -> Result<()> {
        send_response(
            self.stream,
            ServerToClientMessage::Store { scope, key, value },
        )
        .await
    }

    pub async fn redirect(&mut self, path: String) -> Result<()> {
        send_response(self.stream, ServerToClientMessage::RedirectTo { path }).await
    }
}
