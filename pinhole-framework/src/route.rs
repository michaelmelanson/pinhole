use async_trait::async_trait;
use pinhole_protocol::storage::StateMap;

use crate::{Action, Context, Document, Result};

pub enum Render {
    Document(Document),
    RedirectTo(String),
}

#[async_trait]
pub trait Route: Send + Sync {
    fn path(&self) -> &'static str;
    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> Result<()>;
    async fn render(&self, storage: &StateMap) -> Render;
}
