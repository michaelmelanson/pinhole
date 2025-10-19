use pinhole::{
    Action, ButtonProps, ContainerProps, Context, Direction, Document, InputProps, Node, Render,
    Result, Route, StateMap, StateValue, StorageScope, TextProps,
};

use crate::stylesheet::stylesheet;

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
        if storage
            .get("saved_email")
            .unwrap_or(&StateValue::Empty)
            .string()
            != ""
        {
            return Render::RedirectTo("/todos".to_string());
        }

        Render::Document(signin(storage))
    }
}

fn signin(storage: &StateMap) -> Document {
    let saved_email = storage.get("saved_email").map(|v| v.string()).unwrap_or("");

    Document {
        node: Node::Container(ContainerProps {
            direction: Direction::Vertical,
            classes: vec!["login-container".to_string()],
            children: vec![
                Node::Text(TextProps {
                    text: "To-do MVC".to_string(),
                    classes: vec!["title".to_string()],
                }),
                Node::Input(InputProps {
                    label: "Email".to_string(),
                    id: "email".to_string(),
                    password: false,
                    placeholder: if saved_email.is_empty() {
                        Some("yourname@example.com".to_string())
                    } else {
                        Some(saved_email.to_string())
                    },
                    input_classes: vec!["input".to_string()],
                    label_classes: vec![],
                }),
                Node::Input(InputProps {
                    label: "Password".to_string(),
                    id: "password".to_string(),
                    password: true,
                    placeholder: None,
                    input_classes: vec!["input".to_string()],
                    label_classes: vec![],
                }),
                Node::Button(ButtonProps {
                    label: "Sign in".to_string(),
                    on_click: Action::named(
                        SUBMIT_ACTION,
                        vec!["email".to_string(), "password".to_string()],
                    ),
                    classes: vec!["primary-action".to_string()],
                }),
            ],
        }),
        stylesheet: stylesheet(),
    }
}
