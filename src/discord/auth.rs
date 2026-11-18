use crate::discord::{Context, Error};

pub async fn authorize(ctx: Context<'_>) -> Result<(), Error> {
    let application = ctx.http().get_current_application_info().await?;

    let owner = application.owner.ok_or("無法取得 Bot Application Owner")?;

    if ctx.author().id != owner.id {
        return Err("Permission denied".into());
    }

    Ok(())
}
