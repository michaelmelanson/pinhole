use pinhole::{Action, Route, Document, Node, Context, Scope, Storage, Result, Render};

pub struct IndexRoute;

const SUBMIT_ACTION: &str = "submit";

#[async_trait::async_trait]
impl Route for IndexRoute {
  fn path(&self) -> String { "/".to_string() }

  async fn action<'a>(&self, action: &Action, context: &'a mut Context<'a>) -> Result<()> {

    match action {
      Action { name, args: _ } if name == SUBMIT_ACTION => {
        log::info!("Submit with form state: {:?}", context.form_state);

        context.store(Scope::Session, "authenticated".to_string(), "1".to_string()).await?;
        context.redirect("/todos".to_string()).await?;
      },

      _ => log::error!("Unknown action: {:?}", action)
    }

    Ok(())
  }

  async fn render(&self, storage: &Storage) -> Render {
    if storage.get("authenticated").is_some() {
      return Render::RedirectTo("/todos".to_string());
    }

    Render::Document(signin())
  }
}

fn signin() -> Document {
  Document(
    Node::Container {
      children: vec![
        Node::Text { text: "TODO MVC".to_string() }.boxed(),
        Node::Input { 
          label: "Email".to_string(),
          id: "email".to_string(), 
          password: false
        }.boxed(),

        Node::Input { 
          label: "Password".to_string(),
          id: "password".to_string(), 
          password: true
        }.boxed(),

        Node::Button { 
          label: "Sign in".to_string(),
          on_click: Action::named(SUBMIT_ACTION)
        }.boxed()
      ]
    }
  )
}