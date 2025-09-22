use maplit::hashmap;

use pinhole::{
    Action, ButtonProps, CheckboxProps, Context, Document, Layout, Node, Render, Result, Route,
    Size, Sizing, StateMap, StorageScope, TextProps,
};

use crate::model::Todo;

pub struct ListRoute;

const TASK_CHECKED: &str = "checked";
const TASK_ID_KEY: &str = "id";
const LOGOUT_ACTION: &str = "logout";

#[async_trait::async_trait]
impl Route for ListRoute {
    fn path(&self) -> &'static str {
        "/todos"
    }

    async fn action<'a>(&self, action: &Action, context: &mut Context<'a>) -> Result<()> {
        match action {
            Action { name, args, .. } if name == TASK_CHECKED => {
                if let Some(id) = args.get(TASK_ID_KEY) {
                    if let Some(value) = context.storage.get(id) {
                        if value.boolean() {
                            log::info!("Task {:?} checked", id);
                        } else {
                            log::info!("Task {:?} unchecked", id);
                        }
                    }
                }
            }

            Action { name, .. } if name == LOGOUT_ACTION => {
                log::info!("Logging out");

                // Clear saved email (logout)
                context
                    .store(StorageScope::Persistent, "saved_email", "")
                    .await?;

                context.redirect("/").await?;
            }

            _ => log::error!("Unknown action: {:?}", action),
        }

        Ok(())
    }

    async fn render(&self, storage: &StateMap) -> Render {
        // Check authentication - must have saved email
        if storage.get("saved_email").is_none() {
            return Render::RedirectTo("/".to_string());
        }

        let todos = vec![
            Todo {
                id: "1".to_string(),
                text: "Dishes".to_string(),
                done: false,
            },
            Todo {
                id: "2".to_string(),
                text: "Put kid to bed".to_string(),
                done: true,
            },
        ];

        Render::Document(list(&todos, storage))
    }
}

fn list(todos: &Vec<Todo>, storage: &StateMap) -> Document {
    let email = storage
        .get("saved_email")
        .map(|v| v.string())
        .unwrap_or("Unknown user");

    Document(Node::Container {
        layout: Layout::default()
            .horizontal(Sizing::default().centred().size(Size::Fill))
            .vertical(Sizing::default().centred().size(Size::Fill)),

        children: vec![
            Node::Container {
                layout: Layout::default().horizontal(Sizing::default().size(Size::Fill)),
                children: vec![
                    Node::Text(TextProps {
                        text: format!("Your todos ({})", email),
                    })
                    .boxed(),
                    Node::Button(ButtonProps {
                        label: "Logout".to_string(),
                        on_click: Action::named(LOGOUT_ACTION, vec![]),
                    })
                    .boxed(),
                ],
            }
            .boxed(),
            Node::Container {
                layout: Layout::default(),
                children: todos
                    .iter()
                    .map(|t| {
                        Node::Checkbox(CheckboxProps {
                            id: t.id.clone(),
                            label: t.text.clone(),
                            checked: t.done,
                            on_change: Action::new(
                                TASK_CHECKED,
                                hashmap! { TASK_ID_KEY.to_string() => t.id.clone() },
                                vec![t.id.clone()],
                            ),
                        })
                        .boxed()
                    })
                    .collect::<Vec<_>>(),
            }
            .boxed(),
        ],
    })
}
