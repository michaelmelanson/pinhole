use maplit::hashmap;

use pinhole::{
    Action, ButtonProps, CheckboxProps, ContainerProps, Context, Direction, Document, Node, Render,
    Result, Route, StateMap, StorageScope, TextProps,
};

use crate::{model::Todo, stylesheet::stylesheet};

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
                            tracing::debug!(id = %id, "Task checked");
                        } else {
                            tracing::debug!(id = %id, "Task unchecked");
                        }
                    }
                }
            }

            Action { name, .. } if name == LOGOUT_ACTION => {
                tracing::info!("User logged out");

                // Clear saved email (logout)
                context
                    .store(StorageScope::Persistent, "saved_email", "")
                    .await?;

                context.redirect("/").await?;
            }

            _ => tracing::warn!(action = %action.name, "Unknown action"),
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

    Document {
        node: Node::Container(ContainerProps {
            direction: Direction::Vertical,
            children: vec![
                Node::Container(ContainerProps {
                    direction: Direction::Horizontal,
                    children: vec![
                        Node::Container(ContainerProps {
                            direction: Direction::Vertical,
                            children: vec![Node::Text(TextProps {
                                text: "Your todos".to_string(),
                                classes: vec!["title".to_string()],
                            })],
                            classes: vec!["title-container".to_string()],
                        }),
                        Node::Container(ContainerProps {
                            direction: Direction::Vertical,
                            children: vec![
                                Node::Text(TextProps {
                                    text: format!("Welcome, {}", email),
                                    classes: vec![],
                                }),
                                Node::Button(ButtonProps {
                                    label: "Logout".to_string(),
                                    on_click: Action::named(LOGOUT_ACTION, vec![]),
                                    classes: vec!["destructive-action".to_string()],
                                }),
                            ],
                            classes: vec!["account-info".to_string()],
                        }),
                    ],
                    classes: vec!["header-container".to_string()],
                }),
                Node::Container(ContainerProps {
                    direction: Direction::Vertical,
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
                                classes: vec![],
                            })
                        })
                        .collect::<Vec<_>>(),
                    classes: vec!["todo-list".to_string()],
                }),
            ],
            classes: vec![],
        }),
        stylesheet: stylesheet(),
    }
}
