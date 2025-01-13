use std::{default, num::NonZero};

use hl_spray_size_calculator::SetDifference;
use image::{GenericImage, GenericImageView, ImageEncoder, Rgba};
use teloxide::{dispatching::dialogue::{GetChatId, InMemStorage}, net::Download, prelude::*, types::{Document, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, Me, MessageId, MessageKind}};
use vector2::Vector2;

type ConverterDialogue = Dialogue<ConverterState, InMemStorage<ConverterState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum ConverterState {
    #[default]
    Start,
    ReceiveImage,
    ReceiveGameChoice {
        image_original: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
    },
    ReceiveFitChoice {
        image_original: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
        best_fit: hl_spray_size_calculator::RatioFittingResult,
        biggest_fit: hl_spray_size_calculator::RatioFittingResult,
    },
    ReceiveAnchorPoints {
        image_original: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
        ratio_fit: hl_spray_size_calculator::RatioFittingResult,
    },
    ReceiveSize {
        canvas: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
        ratio_fit: hl_spray_size_calculator::RatioFittingResult,
    },
    ReceiveOpacityBound {
        canvas: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("[OK] {:?} -- Bot started", std::time::SystemTime::now());

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<ConverterState>, ConverterState>()
            .branch(dptree::case![ConverterState::Start].endpoint(start))
            .branch(
                Message::filter_document()
                .chain(dptree::case![ConverterState::ReceiveImage].endpoint(receive_image))
            )
            .branch(
                Message::filter_text()
                .chain(dptree::case![ConverterState::ReceiveSize { canvas, ratio_fit }].endpoint(receive_size))
            )
            .branch(
                Message::filter_text()
                .chain(dptree::case![ConverterState::ReceiveOpacityBound { canvas }].endpoint(receive_opacity_bound))
            )
        )
        .branch(
            Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, InMemStorage<ConverterState>, ConverterState>()
            .branch(dptree::case![ConverterState::ReceiveGameChoice { image_original }].endpoint(receive_game_choice))
            .branch(dptree::case![ConverterState::ReceiveFitChoice { image_original, best_fit, biggest_fit }].endpoint(receive_fit_choice))
            .branch(dptree::case![ConverterState::ReceiveAnchorPoints { image_original, ratio_fit }].endpoint(receive_anchor_points))
        );

    Dispatcher::builder(
        bot, 
        handler
    )
    .dependencies(dptree::deps![InMemStorage::<ConverterState>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(bot: Bot, dialogue: ConverterDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "This bot can prepare images to be turned into a GoldSrc spray. To begin, send an image as a document.").await?;
    bot.send_message(msg.chat.id, "Note that the image will be worked on in 16-bit RGBA.").await?;
    dialogue.update(ConverterState::ReceiveImage).await?;
    Ok(())
}

async fn ask_for_game(bot: &Bot, id: ChatId) -> HandlerResult {
    let kb = vec![
        vec![
            InlineKeyboardButton::callback("Half-Life", "Half-Life"),
            InlineKeyboardButton::callback("Sven-Coop", "Sven-Coop")
        ]
    ];
    let kb = InlineKeyboardMarkup::new(kb);
    bot.send_message(id, "Please select a game to make a spray for.").reply_markup(kb).await?;

    Ok(())
}

async fn ask_for_fit(bot: &Bot, id: ChatId) -> HandlerResult {
    let kb = vec![
        vec![
            InlineKeyboardButton::callback("`best`", "best"),
            InlineKeyboardButton::callback("`biggest`", "biggest")
        ]
    ];
    let kb = InlineKeyboardMarkup::new(kb);
    bot.send_message(id, "Please select one of the fitting variants.").reply_markup(kb).await?;

    Ok(())
}

async fn ask_for_anchor(bot: &Bot, id: ChatId, original_size: Vector2<u32>, canvas_size: Vector2<u32>) -> Result<Vec<Vec<InlineKeyboardButton>>, Box<dyn std::error::Error + Send + Sync>> {
    let (horizontal_cmp, vertical_cmp) = original_size.apply_two(canvas_size, |x, y| x.cmp(&y)).into();
    let kb = match (horizontal_cmp, vertical_cmp) {
        (
            std::cmp::Ordering::Less | std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less | std::cmp::Ordering::Greater
        ) =>
            vec![
            vec![
                InlineKeyboardButton::callback("┏", "top left"),
                InlineKeyboardButton::callback("━", "top center"),
                InlineKeyboardButton::callback("┓", "top right"),
            ],
            vec![
                InlineKeyboardButton::callback("┃", "center left"),
                InlineKeyboardButton::callback("╋", "center center"),
                InlineKeyboardButton::callback("┃", "center right"),
            ],
            vec![
                InlineKeyboardButton::callback("┗", "bottom left"),
                InlineKeyboardButton::callback("━", "bottom center"),
                InlineKeyboardButton::callback("┛", "bottom right"),
            ],
            ],
        (std::cmp::Ordering::Less | std::cmp::Ordering::Greater, std::cmp::Ordering::Equal) =>
            vec![
            vec![
                InlineKeyboardButton::callback("┃", "center left"),
                InlineKeyboardButton::callback("╋", "center center"),
                InlineKeyboardButton::callback("┃", "center right"),
            ],
            ],
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Less | std::cmp::Ordering::Greater) =>
            vec![
            vec![
                InlineKeyboardButton::callback("━", "top center"),
            ],
            vec![
                InlineKeyboardButton::callback("╋", "center center"),
            ],
            vec![
                InlineKeyboardButton::callback("━", "bottom center"),
            ],
            ],
        (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal) => {
            vec![]
        },
    };

    match horizontal_cmp {
        std::cmp::Ordering::Less => {
            bot.send_message(id, "Horizontal axis sets image position on future canvas.").await?;
        },
        std::cmp::Ordering::Equal => { },
        std::cmp::Ordering::Greater => {
            bot.send_message(id, "Horizontal axis sets crop position.").await?;
        },
    }
    match vertical_cmp {
        std::cmp::Ordering::Less => {
            bot.send_message(id, "Vertical axis sets image position on future canvas.").await?;
        },
        std::cmp::Ordering::Equal => { },
        std::cmp::Ordering::Greater => {
            bot.send_message(id, "Vertical axis sets crop position.").await?;
        },
    }

    Ok(kb)
}

async fn ask_for_size(bot: &Bot, ratio_fit: hl_spray_size_calculator::RatioFittingResult) {
    
}

async fn receive_image(bot: Bot, dialogue: ConverterDialogue, msg: Message, doc: Document) -> HandlerResult {
    bot.send_message(msg.chat.id, "Fetching the document data…").await?;
    let file_data = bot.get_file(&doc.file.id).await;
    if file_data.is_err() {
        bot.send_message(msg.chat.id, "Could not find the document. Try again?").await?;
        return Ok(());
    }
    let file_data = file_data.unwrap();
    bot.send_message(msg.chat.id, "Document found.").await?;

    let mut image_raw: Vec<u8> = vec![];
    if bot.download_file(&file_data.path, &mut image_raw).await.is_err() {
        bot.send_message(msg.chat.id, "Could not download the document. Try again?").await?;
        return Ok(());
    }
    bot.send_message(msg.chat.id, "Document received.").await?;

    let spray_original =
        image::ImageReader::new(std::io::Cursor::new(image_raw)).with_guessed_format().unwrap().decode();
    if let Ok(original_image) = spray_original {
        bot.send_message(msg.chat.id, "Recognised as image, moving on to next step!").await?;
        if original_image.width() == 0 || original_image.height() == 0 {
            bot.send_message(msg.chat.id, "Image size invalid (has zero), please try a different image.").await?;
            return Ok(());
        }

        ask_for_game(&bot, msg.chat.id).await?;

        dialogue.update(ConverterState::ReceiveGameChoice { image_original: original_image.into_rgba16() }).await?;
    } else {
        bot.send_message(msg.chat.id, "The document could not be interpreted as an image. Check for data corruption.").await?;
    }

    Ok(())
}

async fn receive_game_choice(bot: Bot, dialogue: ConverterDialogue, image_original: image::ImageBuffer<Rgba<u16>, Vec<u16>>, q: CallbackQuery) -> HandlerResult {
    let msg = q.regular_message();
    let msg = match msg {
        Some(m) => Ok(m),
        None => Err("Nothing selected")
    }?;

    let choice = q.data.clone();
    if choice.is_none() {
        bot.send_message(msg.chat.id, "You must choose either Half-Life or Sven-Coop.").await?;

        ask_for_game(&bot, msg.chat.id).await?;

        return Ok(());
    }
    let choice = choice.unwrap();
    let mut game_data = match choice.as_str() {
        "Half-Life" => hl_spray_size_calculator::GameData::new(
            hl_spray_size_calculator::GOLDSRC_MULTIPLE, hl_spray_size_calculator::HALF_LIFE_RELATIVE_SIZE
        ),
        "Sven-Coop" => hl_spray_size_calculator::GameData::new(
            hl_spray_size_calculator::GOLDSRC_MULTIPLE, hl_spray_size_calculator::SVEN_COOP_RELATIVE_SIZE
        ),
        _ => {
            bot.send_message(msg.chat.id, "You must choose either Half-Life or Sven-Coop.").await?;

            ask_for_game(&bot, msg.chat.id).await?;

            return Ok(());
        }
    };
    bot.edit_message_text(msg.chat.id, msg.id, format!("{} selected.", choice)).await?;

    let original_size = Vector2::new(image_original.width(), image_original.height());
    let original_size_nonzero = original_size.map(|x| NonZero::new(x).unwrap());

    let best_fit = game_data.fit_size(original_size_nonzero);
    game_data = game_data.biggest_ratios();
    let biggest_fit = game_data.fit_size(original_size_nonzero);
    {
        let best_fit_ratio = best_fit.ratio().map(|x| x.get());
        let biggest_fit_ratio = biggest_fit.ratio().map(|x| x.get());
        let best_fit_size = best_fit.as_is().map(|x| x.get());
        let biggest_fit_size = biggest_fit.as_is().map(|x| x.get());
        let best_fit_spray = best_fit.resized().last().unwrap().map(|x| x.get());
        let biggest_fit_spray = biggest_fit.resized().last().unwrap().map(|x| x.get());

        bot.send_message(
            msg.chat.id,
            format!(
                "The original image has a size of {}×{}. You can either select the `best` fit or the `biggest` fit.",
                original_size.x, original_size.y
            )
        ).await?;

        let (a, b) = original_size.set_difference(best_fit_size);
        bot.send_message(
            msg.chat.id,
            format!(
                "`best` fit has a {}:{} ratio and requires recanvasing the image to {}×{}.\nThe image will lose {} px ({}% of original) and gain {} px of transparency ({}% of original).\n`best` fit offers a maximum spray size of {}×{}.",
                best_fit_ratio.x, best_fit_ratio.y, best_fit_size.x, best_fit_size.y,
                a, 100.0 * a as f64 / original_size.fold(|x, y| x*y) as f64,
                b, 100.0 * b as f64 / original_size.fold(|x, y| x*y) as f64,
                best_fit_spray.x, best_fit_spray.y
            )
        ).await?;

        let (a, b) = original_size.set_difference(biggest_fit_size);
        bot.send_message(
            msg.chat.id,
            format!(
                "`biggest` fit has a {}:{} ratio and requires recanvasing the image to {}×{}.\nThe image will lose {} px ({}% of original) and gain {} px of transparency ({}% of original).\n`biggest` fit offers a maximum spray size of {}×{}.",
                biggest_fit_ratio.x, biggest_fit_ratio.y, biggest_fit_size.x, biggest_fit_size.y,
                a, 100.0 * a as f64 / original_size.fold(|x, y| x*y) as f64,
                b, 100.0 * b as f64 / original_size.fold(|x, y| x*y) as f64,
                biggest_fit_spray.x, biggest_fit_spray.y
            )
        ).await?;
    }

    ask_for_fit(&bot, msg.chat.id).await?;
            
    dialogue.update(ConverterState::ReceiveFitChoice { image_original, best_fit, biggest_fit }).await?;

    Ok(())
}

async fn receive_fit_choice(
    bot: Bot,
    dialogue: ConverterDialogue,
    (image_original, best_fit, biggest_fit): (image::ImageBuffer<Rgba<u16>, Vec<u16>>, hl_spray_size_calculator::RatioFittingResult, hl_spray_size_calculator::RatioFittingResult),
    q: CallbackQuery
) -> HandlerResult {
    let msg = q.regular_message();
    let msg = match msg {
        Some(m) => Ok(m),
        None => Err("Nothing selected")
    }?;

    let choice = q.data.clone();
    if choice.is_none() {
        bot.send_message(msg.chat.id, "You must choose either `best` or `biggest`.").await?;

        ask_for_fit(&bot, msg.chat.id).await?;

        return Ok(());
    }
    let choice = choice.unwrap();

    let ratio_fit = match choice.as_str() {
        "best" => best_fit,
        "biggest" => biggest_fit,
        _ => {
            bot.send_message(msg.chat.id, "You must choose either `best` or `biggest`.").await?;

            ask_for_fit(&bot, msg.chat.id).await?;

            return Ok(());
        }
    };
    bot.edit_message_text(msg.chat.id, msg.id, format!("`{}` selected.", choice)).await?;

    let original_size = Vector2::new(image_original.width(), image_original.height());
    let canvas_size = ratio_fit.as_is().map(|x| x.get());

    let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

    if kb.is_empty() {
        bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
        let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
        for (size, index) in ratio_fit.resized().iter().zip(1..){
            ratio_msg += format!(
                "{}. {}×{}.\n",
                index, size.x, size.y
            ).as_str();
        }
        ratio_msg += "Select one by sending its number.";
        bot.send_message(msg.chat.id, ratio_msg).await?;

        dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
    } else {
        let kb = InlineKeyboardMarkup::new(kb);
        bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
        dialogue.update(ConverterState::ReceiveAnchorPoints { image_original, ratio_fit }).await?;
    }

    Ok(())
}

async fn receive_anchor_points(
    bot: Bot,
    dialogue: ConverterDialogue,
    (image_original, ratio_fit): (image::ImageBuffer<Rgba<u16>, Vec<u16>>, hl_spray_size_calculator::RatioFittingResult),
    q: CallbackQuery,
) -> HandlerResult {
    let msg = q.regular_message();
    let msg = match msg {
        Some(m) => Ok(m),
        None => Err("Nothing selected")
    }?;

    let original_size = Vector2::new(image_original.width(), image_original.height());
    let canvas_size = ratio_fit.as_is().map(|x| x.get());
    let crop_size = original_size.apply_two(canvas_size, |x, y| x.min(y));

    let choice = q.data.clone();
    if choice.is_none() {
        bot.send_message(msg.chat.id, "You must choose an anchor point.").await?;

        let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

        if kb.is_empty() {
            bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
            let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
            for (size, index) in ratio_fit.resized().iter().zip(1..){
                ratio_msg += format!(
                    "{}. {}×{}.\n",
                    index, size.x, size.y
                ).as_str();
            }
            ratio_msg += "Select one by sending its number.";
            bot.send_message(msg.chat.id, ratio_msg).await?;
            dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
        } else {
            let kb = InlineKeyboardMarkup::new(kb);
            bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
        }

        return Ok(());
    }
    let choice_str = choice.unwrap();
    bot.edit_message_text(msg.chat.id, msg.id, format!("`{}` selected.", choice_str)).await?;

    let choice: Vec<&str> = choice_str.split(" ").collect();
    if choice.len() != 2 {
        bot.send_message(msg.chat.id, "Choice must include both anchor points.").await?;

        let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

        if kb.is_empty() {
            bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
            let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
            for (size, index) in ratio_fit.resized().iter().zip(1..){
                ratio_msg += format!(
                    "{}. {}×{}.\n",
                    index, size.x, size.y
                ).as_str();
            }
            ratio_msg += "Select one by sending its number.";
            bot.send_message(msg.chat.id, ratio_msg).await?;
            dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
        } else {
            let kb = InlineKeyboardMarkup::new(kb);
            bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
        }

        return Ok(());
    }

    let (horizontal_cmp, vertical_cmp) = original_size.apply_two(canvas_size, |x, y| x.cmp(&y)).into();
    let mut canvas = image::ImageBuffer::<Rgba<u16>, Vec<u16>>::new(canvas_size.x, canvas_size.y);
    let spray_original_cropped = {
        let (x, y) = (match horizontal_cmp {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => match choice[1] {
                "left" => 0,
                "center" => (original_size.x - canvas_size.x) / 2,
                "right" => original_size.x - canvas_size.x,
                _ => {
                    bot.send_message(msg.chat.id, "Choice must include proper anchor points.").await?;

                    let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

                    if kb.is_empty() {
                        bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
                        let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
                        for (size, index) in ratio_fit.resized().iter().zip(1..){
                            ratio_msg += format!(
                                "{}. {}×{}.\n",
                                index, size.x, size.y
                            ).as_str();
                        }
                        ratio_msg += "Select one by sending its number.";
                        bot.send_message(msg.chat.id, ratio_msg).await?;
                        dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
                    } else {
                        let kb = InlineKeyboardMarkup::new(kb);
                        bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
                    }

                    return Ok(());
                }
            }
        },
        match vertical_cmp {
            std::cmp::Ordering::Less | std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => match choice[0] {
                "top" => 0,
                "center" => (original_size.y - canvas_size.y) / 2,
                "bottom" => original_size.y - canvas_size.y,
                _ => {
                    bot.send_message(msg.chat.id, "Choice must include proper anchor points.").await?;

                    let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

                    if kb.is_empty() {
                        bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
                        let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
                        for (size, index) in ratio_fit.resized().iter().zip(1..){
                            ratio_msg += format!(
                                "{}. {}×{}.\n",
                                index, size.x, size.y
                            ).as_str();
                        }
                        ratio_msg += "Select one by sending its number.";
                        bot.send_message(msg.chat.id, ratio_msg).await?;
                        dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
                    } else {
                        let kb = InlineKeyboardMarkup::new(kb);
                        bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
                    }

                    return Ok(());
                }
            }
        }
        );

        image_original.view(x, y, crop_size.x, crop_size.y)
    };

    let mut canvas_view = {
        let (x, y) = (match horizontal_cmp {
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Less => match choice[1] {
                "left" => 0,
                "center" => (canvas_size.x - original_size.x) / 2,
                "right" => canvas_size.x - original_size.x,
                _ => {
                    bot.send_message(msg.chat.id, "Choice must include proper anchor points.").await?;

                    let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

                    if kb.is_empty() {
                        bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
                        let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
                        for (size, index) in ratio_fit.resized().iter().zip(1..){
                            ratio_msg += format!(
                                "{}. {}×{}.\n",
                                index, size.x, size.y
                            ).as_str();
                        }
                        ratio_msg += "Select one by sending its number.";
                        bot.send_message(msg.chat.id, ratio_msg).await?;
                        dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
                    } else {
                        let kb = InlineKeyboardMarkup::new(kb);
                        bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
                    }

                    return Ok(());
                }
            }
        },
        match vertical_cmp {
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Less => match choice[0] {
                "top" => 0,
                "center" => (canvas_size.y - original_size.y) / 2,
                "bottom" => canvas_size.y - original_size.y,
                _ => {
                    bot.send_message(msg.chat.id, "Choice must include proper anchor points.").await?;

                    let kb = ask_for_anchor(&bot, msg.chat.id, original_size, canvas_size).await?;

                    if kb.is_empty() {
                        bot.send_message(msg.chat.id, "Image and future canvas share dimensions. Skipping to size selection.").await?;
                        let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
                        for (size, index) in ratio_fit.resized().iter().zip(1..){
                            ratio_msg += format!(
                                "{}. {}×{}.\n",
                                index, size.x, size.y
                            ).as_str();
                        }
                        ratio_msg += "Select one by sending its number.";
                        bot.send_message(msg.chat.id, ratio_msg).await?;
                        dialogue.update(ConverterState::ReceiveSize { canvas: image_original, ratio_fit }).await?;
                    } else {
                        let kb = InlineKeyboardMarkup::new(kb);
                        bot.send_message(msg.chat.id, "Please select an anchor point for cropping/placement.").reply_markup(kb).await?;
                    }

                    return Ok(());
                }
            }
        }
        );

        canvas.sub_image(x, y, crop_size.x, crop_size.y)
    };
    match canvas_view.copy_from(&spray_original_cropped.to_image(), 0, 0) {
        Ok(_) => {},
        Err(_) => unreachable!()
    }

    let mut ratio_msg = format!("{}:{} allows for the following sizes:\n", ratio_fit.ratio().x, ratio_fit.ratio().y);
    for (size, index) in ratio_fit.resized().iter().zip(1..){
        ratio_msg += format!(
            "{}. {}×{}.\n",
            index, size.x, size.y
        ).as_str();
    }
    ratio_msg += "Select one by sending its number.";
    bot.send_message(msg.chat.id, ratio_msg).await?;

    dialogue.update(ConverterState::ReceiveSize { canvas, ratio_fit }).await?;

    Ok(())
}

async fn receive_size(
    bot: Bot,
    dialogue: ConverterDialogue,
    (mut canvas, ratio_fit): (image::ImageBuffer<Rgba<u16>, Vec<u16>>, hl_spray_size_calculator::RatioFittingResult),
    msg: Message
) -> HandlerResult {
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "A text message must be provided.").await?;
        return Ok(());
    }
    let text = msg.text().unwrap().trim();

    let try_parse = text.parse::<usize>();
    if try_parse.is_err() {
        bot.send_message(msg.chat.id, "The message cannot be interpreted as a number.").await?;
        return Ok(());
    }
    let index = try_parse.unwrap();
    if index == 0 || index > ratio_fit.resized().len() {
        bot.send_message(msg.chat.id, "The number does not lie in the list.").await?;
        return Ok(());
    }
    let selected_size = ratio_fit.resized()[index - 1].map(|x| x.get());
    canvas = image::imageops::resize(&canvas, selected_size.x, selected_size.y, image::imageops::Lanczos3);
    bot.send_message(msg.chat.id, "Image resized!").await?;
    bot.send_message(msg.chat.id, format!("Enter the opacity bound ({}-{}). If a pixel's opacity is above that, the pixel is replaced by pure blue; otherwise, the pixel turns completely opaque.", u16::MIN, u16::MAX)).await?;

    dialogue.update(ConverterState::ReceiveOpacityBound { canvas }).await?;

    Ok(())
}

async fn receive_opacity_bound(
    bot: Bot,
    dialogue: ConverterDialogue,
    msg: Message,
    mut canvas: image::ImageBuffer<Rgba<u16>, Vec<u16>>,
) -> HandlerResult {
    if msg.text().is_none() {
        bot.send_message(msg.chat.id, "A text message must be provided.").await?;
        return Ok(());
    }
    let text = msg.text().unwrap().trim();

    let try_parse = text.parse::<u16>();
    if try_parse.is_err() {
        bot.send_message(msg.chat.id, "The message cannot be interpreted as a number.").await?;
        return Ok(());
    }

    let opacity_bound = try_parse.unwrap();
    for px in canvas.pixels_mut() {
        if px.0[3] < u16::MAX - opacity_bound {
            px.0[0] = 0;
            px.0[1] = 0;
            px.0[2] = u16::MAX;
        }
        px.0[3] = u16::MAX;
    }
    let canvas = image::DynamicImage::from(canvas).into_rgb8();
    let mut image_raw: Vec<u8> = vec![];
    match canvas.write_to(&mut std::io::Cursor::new(&mut image_raw), image::ImageFormat::Png) {
        Ok(_) => println!("saved successfully"),
        Err(_) => println!("bad")
    }
    let input_file = InputFile::memory(image_raw).file_name("output.png");
    bot.send_document(msg.chat.id, input_file).await?;
    dialogue.update(ConverterState::Start).await?;

    Ok(())
}
