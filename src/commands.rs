use std::collections::HashMap;
use std::time::Duration;

use regex::Regex;
use serenity::builder::{CreateEmbed, CreateComponents};
use serenity::client::Context;
use serenity::model::interactions::message_component::MessageComponentInteraction;
use serenity::prelude::TypeMapKey;
use serenity::{builder::CreateApplicationCommand};
use serenity::model::interactions::application_command::{ApplicationCommandOptionType, ApplicationCommandInteraction};

use crate::urban_dict::{define, Definition};

pub struct DefinitionCommand;

pub struct DefinitionInteractionData;

impl TypeMapKey for DefinitionInteractionData{
    type Value=HashMap<u64,ApplicationCommandInteraction>;
}

impl DefinitionCommand{
    pub async fn init (ctx:&Context){
        let mut data = ctx.data.write().await;
        data.insert::<DefinitionInteractionData>(HashMap::new());
    }

    pub fn create(command:&mut CreateApplicationCommand) -> &mut CreateApplicationCommand{
        command.name("df");
        command.description("Returns the definition of a word. According to the Urban Dictionary.");
        command.create_option(|options|{
            options.required(true);
            options.name("word");
            options.description("The word to define.");
            options.kind(ApplicationCommandOptionType::String);
            options
        })
    }

    pub async fn handle(ctx:&Context, msg:&ApplicationCommandInteraction)->bool{
        if msg.data.name != "df" {return false;}
        let interaction_id = *msg.id.as_u64();

        let word = msg.data.options.get(0)
            .expect("DefinitionCommand : Expected word option")
            .resolved
            .as_ref()
            .expect("DefinitionCommand : Expected word option to be resolved");

        let word = match word {
            serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue::String(word) => word,
            _ => {
                panic!("DefinitionCommand : Expected word option to be a string");
            },
        }.clone();

        let defs = define(word.as_str()).await.map_err(|err|{
            eprintln!("DefinitionCommand : Error getting definition: {}", err);
        }).unwrap_or_default();
        
        msg.create_interaction_response(ctx, move |resp|{
            resp.interaction_response_data(move |data|{
                data.set_embed(Self::defintion_embed(defs.get(0)));
                data.components(|comp|Self::page_componnent(comp, interaction_id, word.as_str(), 0, defs.len()));
                data
            });
            
            resp
        }).await.expect("DefinitionCommand : Error creating response");
        
        ctx.data.write().await.get_mut::<DefinitionInteractionData>().unwrap().insert(interaction_id, msg.clone());

        tokio::time::sleep(Duration::from_secs(600)).await;

        Self::timeout_interaction(ctx, interaction_id).await;

        true
    }

    pub async fn handle_msg(ctx:&Context, msg:&MessageComponentInteraction)->bool{
        match msg.data.custom_id.as_str() {
            id if id.starts_with("page#") => {
                let reg = Regex::new(r"page#(.+)#(.+)#(\d+)").unwrap();
                let captures = reg.captures(id).unwrap();
                let interaction_id = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
                let word = captures.get(2).unwrap().as_str();
                let page = captures.get(3).unwrap().as_str().parse::<usize>().unwrap();
                
                let defs = define(word).await.map_err(|err|{
                    eprintln!("DefinitionCommand : Error getting definition: {}", err);
                }).unwrap_or_default();
                
                let lock = ctx.data.read().await;
                let original_interaction = lock.get::<DefinitionInteractionData>().unwrap().get(&interaction_id).unwrap();

                original_interaction.edit_original_interaction_response(ctx, |resp|{
                    resp.set_embed(Self::defintion_embed(defs.get(page)));
                    resp.components(|e|Self::page_componnent(e, interaction_id, word, page, defs.len()));
                    resp
                }).await.expect("DefinitionCommand : Error editing response");

                msg.create_interaction_response(ctx, |res|res.kind(serenity::model::interactions::InteractionResponseType::UpdateMessage)).await.expect("DefinitionCommand : Error creating response");
            }
            _ => {
                println!("{:?}", msg.data.custom_id);
                return false;
            }
        }
        true
    }

    fn defintion_embed(def: Option<&Definition>) -> CreateEmbed{
        match def {
            Some(def) => {
                let mut embed = CreateEmbed::default();
                let def = def.clone();
                let title = def.word();
                let description = def.md_formated_definition();
                let score = format!(":thumbsup: {} / :thumbsdown: {}", def.thumbs_up, def.thumbs_down);

                embed.title(title);
                embed.url(def.permalink);
                embed.description(description);
                embed.field("Score : ", score, false);
                embed
            },
            None => {
                let mut embed = CreateEmbed::default();
                embed.title("No definition found");
                embed
            }
        }
    }

    fn page_componnent<'a>(comp: &'a mut CreateComponents, interaction_id: u64, word:&str, page: usize, total_pages: usize) -> &'a mut CreateComponents{
        comp.create_action_row(|row|{
            row.create_button(|button|{
                button.label("previous");
                button.emoji('◀');
                button.custom_id(format!("page#{interaction_id}#{word}#{}", page.max(1)-1));
                button.disabled(total_pages > 0 && page == 0);
                button
            }).create_button(|button|{
                button.label("next");
                button.emoji('▶');
                button.custom_id(format!("page#{interaction_id}#{word}#{}", page+1));
                button.disabled(total_pages > 0 && page == total_pages - 1);
                button
            })
        });
        comp
    }

    async fn timeout_interaction(ctx:&Context, interaction_id: u64){
        let mut lock = ctx.data.write().await;
        let interactions = lock.get_mut::<DefinitionInteractionData>().unwrap();
        let interaction = interactions.remove(&interaction_id);

        if let Some(interaction) = interaction {
            interaction.edit_original_interaction_response(ctx, |resp|{
                resp.components(|e|e);
                resp
            }).await.expect("DefinitionCommand : Error editing response");
        }

    }
}