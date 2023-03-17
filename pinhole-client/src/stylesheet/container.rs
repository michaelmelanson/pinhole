pub struct ContainerStylesheet;
impl iced::widget::container::StyleSheet for ContainerStylesheet {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            ..Default::default()
        }    
    }
}
