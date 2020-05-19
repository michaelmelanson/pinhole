use async_trait::async_trait;

use crate::{Context, Document, Action, Result};
use std::collections::HashMap;

pub type Storage = HashMap<String, String>;

pub enum Render {
  Document(Document),
  RedirectTo(String)
}

#[async_trait]
pub trait Route: Send + Sync {
  fn path(&self) -> String;
  async fn action<'a>(&self, action: &Action, context: & mut Context<'a>) -> Result<()>;
  async fn render(&self, storage: &Storage) -> Render;
}
