use std::collections::hash_map::Entry;
use std::env;
use std::iter::Iterator;
use std::path::Path;

use futures::StreamExt;
use telegram_bot::{
    Api, CanAnswerInlineQuery, InlineQueryResult, InlineQueryResultCachedSticker, MessageKind,
    Update, UpdateKind,
};
use tokio::signal::ctrl_c;

mod data;
use data::{Data, StickersEntry};

type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

async fn handle_update(api: &Api, app: &mut Data, update: Update) -> AppResult<()> {
    match update.kind {
        UpdateKind::Message(message) => {
            if let MessageKind::Sticker { ref data } = message.kind {
                let file_id = data.file_id.clone();
                match app.stickers.entry(data.file_unique_id.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert(StickersEntry::new(file_id));
                    }
                    Entry::Occupied(ref mut entry) => entry.get_mut().update(file_id),
                };
            }
        }
        UpdateKind::InlineQuery(query) => {
            let mut stickers: Vec<&StickersEntry> = app.stickers.values().collect();
            stickers.sort_by_key(|&entry| entry.last_usage_time);
            api.send(
                query
                    .answer(stickers_to_inline_results(stickers.into_iter().rev()))
                    .cache_time(0),
            )
            .await?;
        }
        _ => (),
    };
    Ok(())
}

fn stickers_to_inline_results<'a>(
    stickers: impl Iterator<Item = &'a StickersEntry>,
) -> Vec<InlineQueryResult> {
    stickers
        .map(|entry| {
            InlineQueryResult::InlineQueryResultCachedSticker(InlineQueryResultCachedSticker {
                id: entry.id.to_string(),
                sticker_file_id: entry.file_id.clone(),
                reply_markup: None,
                input_message_content: None,
            })
        })
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("data.json");
    let mut app = Data::read_from(path)?;

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("TELEGRAM_BOT_TOKEN not set");
    let api = Api::new(token);

    #[cfg(feature = "trace")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter("telegram_bot=trace")
            .finish(),
    )
    .unwrap();

    // Fetch new updates via long poll method
    let mut stream = api.stream();
    loop {
        tokio::select! {
            Some(update) = stream.next() => {
                match update {
                    Ok(update) => {
                        handle_update(&api, &mut app, update).await?;
                    }
                    Err(error) => {
                        eprintln!("Error: {:?}", error);
                    }
                }
            }
            _ = ctrl_c() => {
                break;
            }
        }
    }

    // Exiting
    app.write_to(path)?;
    Ok(())
}
