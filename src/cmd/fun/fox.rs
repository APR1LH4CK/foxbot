use std::{fs, path::Path};

use poise::serenity_prelude::{CreateAttachment, CreateEmbed};
use rand::prelude::*;

use crate::{Context, Error, util::embed};
#[poise::command(slash_command)]
pub async fn fox(ctx: Context<'_>) -> Result<(), Error> {
    let images_dir = Path::new("images/foxes");

    if !images_dir.exists() {
        let embed = embed::create_error_embed("Error", "Fox images directory not found!");
        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let entries =
        fs::read_dir(images_dir).map_err(|e| format!("Failed to read images directory: {}", e))?;

    let mut fox_images = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jpg") {
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                fox_images.push(file_name.to_string());
            }
        }
    }

    if fox_images.is_empty() {
        let embed = embed::create_error_embed("Error", "No fox images found in the directory!");
        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
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

    let image_path = images_dir.join(&selected_image);

    let image_data =
        fs::read(&image_path).map_err(|e| format!("Failed to read image file: {}", e))?;

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
