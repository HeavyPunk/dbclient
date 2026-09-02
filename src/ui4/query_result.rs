use iocraft::{
    component,
    components::{BorderStyle, Text, View},
    element, AlignContent, AnyElement, Color, Props,
};

#[derive(Default, Props)]
pub struct QueryResultProps {}

#[component]
pub fn QueryResult(props: &QueryResultProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: 100pct,
            height: 100pct,
            justify_content: Some(AlignContent::Center),
            border_style: BorderStyle::Round,
            border_color: Color::Cyan,
        ) {
            Text(content: "QueryResult")
        }
    }
}
