use pinhole::{
    Action, ButtonProps, ContainerProps, Context, Direction, Document, InputProps, Node, Params,
    Render, Result, Route, StateMap, TextProps,
};

use crate::stylesheet::stylesheet;

const HOVER_BUTTON_CLICKED: &str = "hover_button_clicked";

pub struct ShowcaseRoute;

#[async_trait::async_trait]
impl Route for ShowcaseRoute {
    fn path(&self) -> &'static str {
        "/"
    }

    async fn action<'a>(
        &self,
        action: &Action,
        _params: &Params,
        _context: &mut Context<'a>,
    ) -> Result<()> {
        match action.name.as_str() {
            HOVER_BUTTON_CLICKED => {
                tracing::info!("Hover button clicked");
            }
            _ => tracing::warn!(action = %action.name, "Unknown action"),
        }

        Ok(())
    }

    async fn render(&self, _params: &Params, _storage: &StateMap) -> Render {
        Render::Document(Document {
            node: page(),
            stylesheet: stylesheet(),
        })
    }
}

fn text(value: impl ToString, classes: &[&str]) -> Node {
    Node::Text(TextProps {
        text: value.to_string(),
        classes: classes.iter().map(|c| c.to_string()).collect(),
    })
}

fn container(direction: Direction, classes: &[&str], children: Vec<Node>) -> Node {
    Node::Container(ContainerProps {
        direction,
        children,
        classes: classes.iter().map(|c| c.to_string()).collect(),
    })
}

/// A flex item: a container with a single centred label, whose own classes
/// carry the `FlexGrow`/height that make the `flex-row`'s `AlignItems`
/// visible.
fn flex_item(class: &str, label: &str) -> Node {
    container(
        Direction::Vertical,
        &[class],
        vec![text(label, &["flex-item-label"])],
    )
}

fn page() -> Node {
    container(
        Direction::Vertical,
        &["page"],
        vec![
            text("Pinhole Styling Showcase", &["title"]),
            text(
                "Resize the window, hover the button, and click into the \
                 text field to see @media, :hover and :focus in action.",
                &["subtitle"],
            ),
            text("Flexbox: AlignItems + FlexGrow", &["section-title"]),
            container(
                Direction::Horizontal,
                &["flex-row"],
                vec![
                    flex_item("flex-item-a", "grow: 1"),
                    flex_item("flex-item-b", "grow: 2"),
                    flex_item("flex-item-c", "grow: 1"),
                ],
            ),
            text("Pseudo-classes: :hover / :focus", &["section-title"]),
            container(
                Direction::Horizontal,
                &["button-row"],
                vec![
                    Node::Button(ButtonProps {
                        label: "Hover me".to_string(),
                        on_click: Action::named(HOVER_BUTTON_CLICKED, vec![]),
                        classes: vec!["hover-button".to_string()],
                    }),
                    Node::Input(InputProps {
                        id: "focus_demo".to_string(),
                        label: "Click in: border goes red \u{2192} blue".to_string(),
                        password: false,
                        placeholder: Some("Focus me".to_string()),
                        label_classes: vec!["subtitle".to_string()],
                        input_classes: vec!["focus-input".to_string()],
                    }),
                ],
            ),
            text("Responsive: @media", &["section-title"]),
            container(
                Direction::Vertical,
                &["media-box"],
                vec![text(
                    "Red under 700px wide, amber 700-1099px, green from 1100px \
                     up \u{2014} drag the window edge to see it change",
                    &["media-box-label"],
                )],
            ),
        ],
    )
}
