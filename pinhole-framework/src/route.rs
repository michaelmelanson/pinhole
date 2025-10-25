use async_trait::async_trait;
use pinhole_protocol::storage::StateMap;

use crate::{Action, Context, Document, Params, Result, RoutePattern};

pub enum Render {
    Document(Document),
    RedirectTo(String),
}

#[async_trait]
pub trait Route: Send + Sync {
    fn path(&self) -> &'static str;

    fn pattern(&self) -> RoutePattern {
        RoutePattern::new(self.path())
    }

    async fn action<'a>(
        &self,
        action: &Action,
        params: &Params,
        context: &mut Context<'a>,
    ) -> Result<()>;

    async fn render(&self, params: &Params, storage: &StateMap) -> Render;
}
