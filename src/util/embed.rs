use poise::serenity_prelude::{Colour, CreateEmbed};

pub const MAIN_COLOR: Colour = Colour::from_rgb(222, 165, 132);

pub fn create_basic_embed() -> CreateEmbed {
    CreateEmbed::default().color(MAIN_COLOR)
}

#[allow(dead_code)]
pub fn create_embed(title: &str, description: &str) -> CreateEmbed {
    create_basic_embed().title(title).description(description)
}

#[allow(dead_code)]
pub fn create_success_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(87, 242, 135))
}

#[allow(dead_code)]
pub fn create_error_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(237, 66, 69))
}
#[allow(dead_code)]
pub fn create_warning_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::from_rgb(255, 221, 51))
}

#[allow(dead_code)]
pub fn create_info_embed(title: &str, description: &str) -> CreateEmbed {
    create_embed(title, description)
}
