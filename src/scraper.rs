use std::thread;
use std::time::Duration;

use config::CONFIG;
use github;
use releases;
use buildbot;

pub fn start_scraping() {
    // spawn the github scraper
    let gh_scraper = thread::spawn(|| {
        let sleep_duration = Duration::from_secs(CONFIG.github_interval_mins * 60);
        loop {
            if let Ok(gh_most_recent) = github::most_recent_update() {
                info!("scraping github activity since {:?}", gh_most_recent);
                match github::ingest_since(gh_most_recent) {
                    Ok(()) => info!("scraped github successfully"),
                    Err(why) => error!("unable to scrape github: {:?}", why),
                }
            } else {
                error!("Unable to determine most recent github update.");
            }

            info!("GitHub scraper sleeping for {} seconds ({} minutes)",
                  sleep_duration.as_secs(),
                  CONFIG.github_interval_mins);
            thread::sleep(sleep_duration);
        }
    });

    // spawn the nightly release scraper
    let rel_scraper = thread::spawn(|| {
        let sleep_duration = Duration::from_secs(CONFIG.release_interval_mins * 60);
        loop {
            if let Ok(rel_most_recent) = releases::most_recent_update() {
                info!("scraping release activity since {:?}", rel_most_recent);
                match releases::ingest_releases_since(rel_most_recent) {
                    Ok(()) => info!("scraped releases successfully"),
                    Err(why) => error!("unable to scrape releases: {:?}", why),
                }
            } else {
                error!("Unable to determine most recent release date.");
            }

            info!("Release scraper sleeping for {} seconds ({} minutes)",
                  sleep_duration.as_secs(),
                  CONFIG.release_interval_mins);
            thread::sleep(sleep_duration);
        }
    });

    // spawn the buildbot scraper
    let bb_scraper = thread::spawn(|| {
        let sleep_duration = Duration::from_secs(CONFIG.buildbot_interval_mins * 60);
        loop {
            info!("scraping all buildbots...");
            match buildbot::ingest() {
                Ok(()) => info!("scraped build status successfully"),
                Err(why) => error!("unable to scrape build status: {:?}", why),
            }

            info!("Buildbot scraper sleeping for {} seconds ({} minutes)",
                  sleep_duration.as_secs(),
                  CONFIG.buildbot_interval_mins);
            thread::sleep(sleep_duration);
        }
    });

    gh_scraper.join().unwrap();
    rel_scraper.join().unwrap();
    bb_scraper.join().unwrap();
}
