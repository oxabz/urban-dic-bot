mod commands;
mod urban_dict;

use std::env;

use serenity::{
    async_trait,
    model::{gateway::Ready, guild::Guild, interactions::Interaction},
    prelude::*,
};

use crate::commands::DefinitionCommand;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        DefinitionCommand::init(&ctx).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        println!("Bot is connected in {} ({} {is_new})", guild.name, guild.id);
        guild
            .id
            .set_application_commands(&ctx.http, |cmd| {
                cmd.create_application_command(DefinitionCommand::create)
            })
            .await
            .expect("Error setting application commands");
    }

    async fn interaction_create(&self, ctx: Context, msg: Interaction) {
        match msg {
            Interaction::ApplicationCommand(cmd) => match cmd {
                _ if DefinitionCommand::handle(&ctx, &cmd).await => {}
                _ => {
                    panic!("Unknown command");
                }
            },
            Interaction::MessageComponent(msg) => match msg {
                _ if DefinitionCommand::handle_msg(&ctx, &msg).await => {}
                _ => {
                    println!("Unhandled message : {:?}", msg);
                }
            },
            _ => {
                println!("Unhandled message type : {:?}", msg);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::empty() | GatewayIntents::GUILDS;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
