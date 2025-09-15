use poise::serenity_prelude::CreateEmbed;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{Context, Error, util::embed};

#[derive(Debug, Deserialize, Serialize)]
struct FoxFact {
    id: u32,
    fact: String,
}

#[poise::command(slash_command)]
pub async fn fact(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    
    let facts_data = match fs::read_to_string("facts.json").await {
        Ok(data) => data,
        Err(_) => {
            let embed = embed::create_error_embed("Error", "Could not load fox facts file!");
            ctx.send(poise::CreateReply::default().embed(embed))
                .await?;
            return Ok(());
        }
    };

    let facts: Vec<FoxFact> = match serde_json::from_str(&facts_data) {
        Ok(facts) => facts,
        Err(_) => {
            let embed = embed::create_error_embed("Error", "Could not parse fox facts data!");
            ctx.send(poise::CreateReply::default().embed(embed))
                .await?;
            return Ok(());
        }
    };

    if facts.is_empty() {
        let embed = embed::create_error_embed("Error", "No fox facts available!");
        ctx.send(poise::CreateReply::default().embed(embed))
            .await?;
        return Ok(());
    }

    let selected_fact = {
        let mut rng = rand::rng();
        facts
            .choose(&mut rng)
            .ok_or("Failed to select random fact")?
    };

    let embed = create_fact_embed(&selected_fact.fact, selected_fact.id);

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
