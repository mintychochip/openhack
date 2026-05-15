use crate::interaction::InteractionContext;

/// Handle the `/openhack help` command.
///
/// # Expected Behavior
///
/// Displays a rich embed listing all available OpenHack bot commands.
/// Does not require a linked account. Returns a type-4 interaction
/// response with an embed.
///
/// # Errors
///
/// None. Always succeeds.
///
/// # Side Effects
///
/// None. Returns a response; no I/O.
pub async fn handle(_ctx: &InteractionContext) -> serde_json::Value {
    let embed = super::make_embed(
        "OpenHack Bot Commands",
        "Here are all the commands you can use:\n\n\
         `/openhack register` - Link your Discord to your OpenHack account\n\
         `/openhack status` - Show current hackathon status\n\
         `/openhack team-info` - Show your team details\n\
         `/openhack team-create <name>` - Create a new team\n\
         `/openhack team-join <code>` - Join a team with an invite code\n\
         `/openhack submit` - Submit a project (opens a form)\n\
         `/openhack leaderboard [top]` - Show the leaderboard\n\
         `/openhack schedule` - Show upcoming events and phases\n\
         `/openhack help` - Show this help message",
        0x00_99_ff,
    );

    super::embed_response(embed)
}
