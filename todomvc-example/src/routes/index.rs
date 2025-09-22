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

                // Store email persistently for authentication
                let email_value = context.storage.get("email").map(|e| e.string().to_string());
                if let Some(email) = email_value {
                    context
                        .store(StorageScope::Persistent, "saved_email", email)
                        .await?;
                }

                context.redirect("/todos").await?;
            }

            _ => log::error!("Unknown action: {:?}", action),
        }

        Ok(())
    }

    async fn render(&self, storage: &StateMap) -> Render {
        // Auto-login if email is already saved
        if storage.get("saved_email").is_some() {
            return Render::RedirectTo("/todos".to_string());
        }

        Render::Document(signin(storage))
    }
}

fn signin(storage: &StateMap) -> Document {
    let saved_email = storage.get("saved_email").map(|v| v.string()).unwrap_or("");

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
                placeholder: if saved_email.is_empty() {
                    Some("yourname@example.com".to_string())
                } else {
                    Some(saved_email.to_string())
                },
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
