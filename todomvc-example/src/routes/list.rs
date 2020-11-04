use maplit::hashmap;

use pinhole::{
    Action, CheckboxProps, Context, Document, Node, Render, Result, Route, Storage, TextProps,
};

use crate::model::Todo;

pub struct ListRoute;

const TODO_CHECKED: &str = "checked";
const ID_KEY: &str = "id";

#[async_trait::async_trait]
impl Route for ListRoute {
    fn path(&self) -> String {
        "/todos".to_string()
    }

    async fn action<'a>(&self, action: &Action, _context: &mut Context<'a>) -> Result<()> {
        match action {
            Action { name, args } if name == TODO_CHECKED => {
                log::info!("Task {:?} checked", args.get(ID_KEY));
            }

            _ => log::error!("Unknown action: {:?}", action),
        }

        Ok(())
    }

    async fn render(&self, _storage: &Storage) -> Render {
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

        Render::Document(list(&todos))
    }
}

fn list(todos: &Vec<Todo>) -> Document {
    Document(Node::Container {
        children: vec![
            Node::Text(TextProps {
                text: "Your todos".to_string(),
            })
            .boxed(),
            Node::Container {
                children: todos
                    .iter()
                    .map(|t| {
                        let action = Action::new(
                            TODO_CHECKED,
                            hashmap! { ID_KEY.to_string() => t.id.clone() },
                        );
                        Node::Checkbox(CheckboxProps {
                            id: t.id.clone(),
                            label: t.text.clone(),
                            checked: t.done,
                            on_change: action,
                        })
                        .boxed()
                    })
                    .collect::<Vec<_>>(),
            }
            .boxed(),
        ],
    })
}
