use pinhole::{
    Action, ButtonProps, Context, Document, InputProps, Layout, Node, Render, Result, Route, Size,
    Sizing, StateMap, StorageScope, TextProps,
};

pub struct IndexRoute;

const SUBMIT_ACTION: &str = "submit";

#[async_trait::async_trait]
impl Route for IndexRoute {
    fn path(&self) -> &'static str {
        "/"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> Result<()> {
        match action {
            Action { name, .. } if name == SUBMIT_ACTION => {
                log::info!("Submit with state: {:?}", context.storage);

                context
                    .store(StorageScope::Session, "authenticated", "1")
                    .await?;
                context.redirect("/todos").await?;
            }

            _ => log::error!("Unknown action: {:?}", action),
        }

        Ok(())
    }

    async fn render(&self, storage: &StateMap) -> Render {
        if storage.get("authenticated").is_some() {
            return Render::RedirectTo("/todos".to_string());
        }

        Render::Document(signin())
    }
}

fn signin() -> Document {
    Document(Node::Container {
        layout: Layout::default()
            .horizontal(Sizing::default().centred().size(Size::Fixed(300)))
            .vertical(Sizing::default().centred().size(Size::Fixed(200))),

        children: vec![
            Node::Text(TextProps {
                text: "TODO MVC".to_string(),
            })
            .boxed(),
            Node::Input(InputProps {
                label: "Email".to_string(),
                id: "email".to_string(),
                password: false,
                placeholder: Some("yourname@example.com".to_string()),
            })
            .boxed(),
            Node::Input(InputProps {
                label: "Password".to_string(),
                id: "password".to_string(),
                password: true,
                placeholder: None,
            })
            .boxed(),
            Node::Button(ButtonProps {
                label: "Sign in".to_string(),
                on_click: Action::named(
                    SUBMIT_ACTION,
                    vec!["email".to_string(), "password".to_string()],
                ),
            })
            .boxed(),
        ],
    })
}
