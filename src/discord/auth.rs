use poise::serenity_prelude as serenity;

use crate::discord::{Context, Error};

pub async fn authorize(ctx: Context<'_>) -> Result<(), Error> {
    let application = ctx.http().get_current_application_info().await?;

    let owner_id = application
        .team
        .as_ref()
        .and_then(|team| {
            team.members
                .iter()
                .find(|member| member.membership_state == serenity::MembershipState::Accepted)
        })
        .map(|member| member.user.id)
        .or_else(|| application.owner.as_ref().map(|owner| owner.id));

    if Some(ctx.author().id) != owner_id {
        return Err("Permission denied".into());
    }

    Ok(())
}
