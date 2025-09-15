use poise::serenity_prelude::{CreateAttachment, CreateEmbed};
use rand::prelude::*;

use crate::{Context, Error, util::embed};

#[poise::command(slash_command)]
pub async fn fox(ctx: Context<'_>) -> Result<(), Error> {
    let fox_images = &ctx.data().fox_images;
    let fox_image_data = &ctx.data().fox_image_data;
    if fox_images.is_empty() {
        let embed = embed::create_error_embed("Error", "No fox images found!");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let selected_image = {
        let mut rng = rand::rng();
        fox_images
            .choose(&mut rng)
            .ok_or("Failed to select random image")?
            .clone()
    };

    let (photographer, image_id) = parse_filename(&selected_image);

    let image_data = fox_image_data
        .get(&selected_image)
        .ok_or("Image data not found in cache")?
        .clone();

    let attachment = CreateAttachment::bytes(image_data, selected_image.clone());

    let embed = create_fox_embed(&photographer, &image_id, &selected_image);

    ctx.send(
        poise::CreateReply::default()
            .embed(embed)
            .attachment(attachment),
    )
    .await?;

    Ok(())
}

fn parse_filename(filename: &str) -> (String, String) {
    let name_without_ext = filename.trim_end_matches(".jpg");

    let parts: Vec<&str> = name_without_ext.split('-').collect();

    if parts.len() >= 3 {
        let id = parts.last().unwrap().to_string();

        let name_parts: Vec<String> = parts[..parts.len() - 1]
            .iter()
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect();

        let photographer = name_parts.join(" ");
        (photographer, id)
    } else {
        (name_without_ext.to_string(), "Unknown".to_string())
    }
}

fn create_fox_embed(photographer: &str, image_id: &str, filename: &str) -> CreateEmbed {
    embed::create_basic_embed()
        .title("Random Fox")
        .description("Here's a beautiful fox for you!")
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(format!(
            "Photo by {} • ID: {}",
            photographer, image_id
        )))
        .image(format!("attachment://{}", filename))
}
