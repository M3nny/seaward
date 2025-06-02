//! Core methods.

use crate::{
    app::utils::{find_links_in_document, find_matches, get_document, print_matches},
    info,
    structs::{crawl_result::CrawlResult, error::AppError, queue_item::QueueItem, state::AppState},
};
use fastbloom::BloomFilter;
use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, Mutex},
};
use tokio::{signal::ctrl_c, sync::mpsc, task::JoinSet};

/// Process a single url and return the result.
///
/// # Parameters
/// - `state`: shared state across multiple tasks.
/// - `url`: url to be crawled.
/// - `depth`: `url` distance relative to the starting url.
async fn process_url(state: Arc<AppState>, url: &str, depth: u32) -> Result<CrawlResult, AppError> {
    if state.cancel_token.is_cancelled() {
        return Err(AppError::cancelled());
    }

    let mut result = CrawlResult {
        url: url.to_string(),
        depth,
        links: HashSet::new(),
        matches: Vec::new(),
    };

    let document = get_document(&state.client, &url).await?;

    result.matches = find_matches(&state.word_selectors, &state.regex, &document);

    // extract links if we haven't reached max depth
    if state.args.depth == 0 || depth < state.args.depth {
        result.links = find_links_in_document(
            &state.args.url,
            &document,
            &state.link_selectors,
            state.args.strict,
        )?;
    }

    Ok(result)
}

/// Cleanup the remaining tasks.
///
/// # Parameters
/// - `active_tasks`: tasks that are currently running.
/// - `rx`: receiver used for clearing the channel.
async fn cleanup_tasks(mut active_tasks: JoinSet<()>, mut rx: mpsc::UnboundedReceiver<Result<CrawlResult, AppError>>) {
    active_tasks.abort_all();
    while active_tasks.join_next().await.is_some() {}
    while rx.try_recv().is_ok() {}
}

/// Spawn concurrent tasks up to the max concurrency limit that has been set.
///
/// # Parameters
/// - `state`: shared state across multiple tasks.
/// - `visited`: bloom filter used to check if a url has already been visited.
/// - `to_visit`: queue used to store the urls that need to be visited.
/// - `active_tasks`: tasks that are currently running.
/// - `tx`: transmitter used for sending crawl results to the consumer.
/// - `pending_results`: number of crawl results that are being processed.
fn spawn_tasks(
    state: &Arc<AppState>,
    visited: &Arc<Mutex<BloomFilter>>,
    to_visit: &mut VecDeque<QueueItem>,
    active_tasks: &mut JoinSet<()>,
    tx: &mpsc::UnboundedSender<Result<CrawlResult, AppError>>,
    pending_results: &mut u32,
) {
    while active_tasks.len() < state.args.concurrency && !to_visit.is_empty() {
        if let Some(QueueItem(current_url, current_depth)) = to_visit.pop_front() {
            // Critical section:
            // after acquiring the mutex, the memership of current_url is checked against the
            // bloom filter, and if it's not statisfied, the url is added to the queue.
            //
            // The mutex is dropped when it goes out of scope.
            {
                let mut visited_guard = visited.lock().unwrap();
                visited_guard.insert(&current_url);
            }

            // spawn task
            let state_clone = state.clone();
            let tx_clone = tx.clone();

            active_tasks.spawn(async move {
                let crawl_result = process_url(state_clone, &current_url, current_depth).await;
                let _ = tx_clone.send(crawl_result);
            });

            *pending_results += 1;
        }
    }
}

/// Crawl entry point.
///
/// # Parameters
/// - `state`: shared state across multiple tasks.
pub async fn crawl(state: Arc<AppState>) -> Result<(), AppError> {
    let visited = Arc::new(Mutex::new(
        BloomFilter::with_false_pos(0.001).expected_items(1000),
    ));
    let mut to_visit = VecDeque::<QueueItem>::from([QueueItem(state.args.url.clone(), 1)]);

    let (tx, mut rx) = mpsc::unbounded_channel::<Result<CrawlResult, AppError>>();
    let mut active_tasks = JoinSet::new();
    let mut pending_results: u32 = 0;

    loop {
        spawn_tasks(
            &state,
            &visited,
            &mut to_visit,
            &mut active_tasks,
            &tx,
            &mut pending_results,
        );

        // exit the crawl loop if the are no active tasks or pending results
        if active_tasks.is_empty() && pending_results == 0 {
            break;
        }

        tokio::select! {
            biased;

            _ = ctrl_c() => {
                state.cancel_token.cancel();
                info!("Shutting down: received keyboard interrupt");
                break;
            }

            Some(_) = active_tasks.join_next() => {}

            Some(crawl_result) = rx.recv() => {
                pending_results -= 1;

                if let Ok(crawl_result) = crawl_result {
                    print_matches(crawl_result.url.as_str(), &state.regex, &crawl_result.matches);
                    let visited_guard = visited.lock().unwrap();
                    crawl_result.links.iter().for_each(|link| {
                        if !visited_guard.contains(&link) {
                            to_visit.push_back(QueueItem(link.to_string(), crawl_result.depth + 1));
                        }
                    });
                }
            }
        }
    }

    cleanup_tasks(active_tasks, rx).await;
    Ok(())
}
