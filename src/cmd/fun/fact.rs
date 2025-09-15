use poise::serenity_prelude::CreateEmbed;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};

use crate::{Context, Error, util::embed};

#[derive(Debug, Deserialize, Serialize)]
struct FoxFact {
    id: u32,
    fact: String,
}

#[poise::command(slash_command)]
pub async fn fact(ctx: Context<'_>) -> Result<(), Error> {
    let facts = &ctx.data().facts;

    if facts.is_empty() {
        let embed = embed::create_error_embed("Error", "No fox facts available!");
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let selected_fact = {
        let mut rng = rand::rng();
        facts
            .choose(&mut rng)
            .ok_or("Failed to select random fact")?
    };

    let fact_text = selected_fact["fact"].as_str().unwrap_or("Unknown fact");
    let fact_id = selected_fact["id"].as_u64().unwrap_or(0) as u32;

    let embed = create_fact_embed(fact_text, fact_id);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn create_fact_embed(fact: &str, fact_id: u32) -> CreateEmbed {
    embed::create_basic_embed()
        .title("Fox Fact")
        .description(fact)
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(format!(
            "Fact #{}",
            fact_id
        )))
}
