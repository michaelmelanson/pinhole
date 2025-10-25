use pinhole::{
    Action, ButtonProps, ContainerProps, Context, Direction, Document, Node, Params, Render,
    Result, Route, StateMap, TextProps,
};

use crate::{model::Todo, stylesheet::stylesheet};

pub struct DetailRoute;

const BACK_ACTION: &str = "back";

#[async_trait::async_trait]
impl Route for DetailRoute {
    fn path(&self) -> &'static str {
        "/todos/:id"
    }

    async fn action<'a>(
        &self,
        action: &Action,
        _params: &Params,
        context: &mut Context<'a>,
    ) -> Result<()> {
        match action {
            Action { name, .. } if name == BACK_ACTION => {
                context.redirect("/todos").await?;
            }
            _ => tracing::warn!(action = %action.name, "Unknown action"),
        }

        Ok(())
    }

    async fn render(&self, params: &Params, storage: &StateMap) -> Render {
        // Check authentication
        if storage.get("saved_email").is_none() {
            return Render::RedirectTo("/".to_string());
        }

        // Get the todo ID from the path parameter
        let todo_id = params.get("id").map(|s| s.as_str()).unwrap_or("");

        // In a real app, this would fetch from a database
        // For now, we'll use the same hardcoded todos
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

        // Find the todo with matching ID
        let todo = todos.iter().find(|t| t.id == todo_id);

        match todo {
            Some(todo) => Render::Document(detail_view(todo)),
            None => Render::RedirectTo("/todos".to_string()),
        }
    }
}

fn detail_view(todo: &Todo) -> Document {
    Document {
        node: Node::Container(ContainerProps {
            direction: Direction::Vertical,
            children: vec![
                Node::Container(ContainerProps {
                    direction: Direction::Horizontal,
                    children: vec![
                        Node::Button(ButtonProps {
                            label: "← Back to list".to_string(),
                            on_click: Action::named(BACK_ACTION, vec![]),
                            classes: vec!["secondary-action".to_string()],
                        }),
                        Node::Text(TextProps {
                            text: "Todo Details".to_string(),
                            classes: vec!["title".to_string()],
                        }),
                    ],
                    classes: vec!["header-container".to_string()],
                }),
                Node::Container(ContainerProps {
                    direction: Direction::Vertical,
                    children: vec![
                        Node::Text(TextProps {
                            text: format!("ID: {}", todo.id),
                            classes: vec![],
                        }),
                        Node::Text(TextProps {
                            text: format!("Task: {}", todo.text),
                            classes: vec![],
                        }),
                        Node::Text(TextProps {
                            text: format!(
                                "Status: {}",
                                if todo.done { "Done" } else { "Not done" }
                            ),
                            classes: vec![],
                        }),
                    ],
                    classes: vec!["detail-info".to_string()],
                }),
            ],
            classes: vec![],
        }),
        stylesheet: stylesheet(),
    }
}
