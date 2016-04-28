use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime};
use diesel::expression::dsl::*;
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::types::{BigInt, Date, Double, Integer, Text};

use DB_POOL;
use error::DashResult;

#[derive(Clone, Debug, Serialize)]
pub struct DashSummary {
    pull_requests: PullRequestSummary,
    issues: IssueSummary,
    buildbots: BuildbotSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct PullRequestSummary {
    opened_per_day: BTreeMap<NaiveDate, i64>,
    closed_per_day: BTreeMap<NaiveDate, i64>,
    merged_per_day: BTreeMap<NaiveDate, i64>,
    num_closed_per_week: BTreeMap<NaiveDate, i64>,
    days_open_before_close: BTreeMap<NaiveDate, f64>,
    current_open_age_days_mean: f64,
    bors_retries: BTreeMap<String, i64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IssueSummary {
    opened_per_day: BTreeMap<NaiveDate, i64>,
    closed_per_day: BTreeMap<NaiveDate, i64>,
    num_closed_per_week: BTreeMap<NaiveDate, i64>,
    days_open_before_close: BTreeMap<NaiveDate, f64>,
    current_open_age_days_mean: f64,
    num_open_p_high_issues: i64,
    num_open_regression_nightly_issues: i64,
    num_open_regression_beta_issues: i64,
    num_open_regression_stable_issues: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildbotSummary {
    per_builder_times_mins: BTreeMap<String, BTreeMap<NaiveDate, f64>>,
    per_builder_failures: BTreeMap<String, BTreeMap<NaiveDate, i64>>,
}

pub fn summary(since: NaiveDate, until: NaiveDate) -> DashResult<DashSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    let pr_open_time = try!(prs_open_time_before_close(since, until));
    let issue_open_time = try!(issues_open_time_before_close(since, until));

    let current_pr_age = try!(open_prs_avg_days_old());
    let current_issue_age = try!(open_issues_avg_days_old());

    let nightly_regress = try!(open_issues_with_label("regression-from-stable-to-nightly"));
    let beta_regress = try!(open_issues_with_label("regression-from-stable-to-beta"));
    let stable_regress = try!(open_issues_with_label("regression-from-stable-to-stable"));

    Ok(DashSummary {
        pull_requests: PullRequestSummary {
            opened_per_day: try!(prs_opened_per_day(since, until)),
            closed_per_day: try!(prs_closed_per_day(since, until)),
            merged_per_day: try!(prs_merged_per_day(since, until)),
            num_closed_per_week: pr_open_time.0,
            days_open_before_close: pr_open_time.1,
            current_open_age_days_mean: current_pr_age,
            bors_retries: try!(bors_retries_per_pr(since, until)),
        },
        issues: IssueSummary {
            opened_per_day: try!(issues_opened_per_day(since, until)),
            closed_per_day: try!(issues_closed_per_day(since, until)),
            num_closed_per_week: issue_open_time.0,
            days_open_before_close: issue_open_time.1,
            current_open_age_days_mean: current_issue_age,
            num_open_p_high_issues: try!(open_issues_with_label("P-high")),
            num_open_regression_nightly_issues: nightly_regress,
            num_open_regression_beta_issues: beta_regress,
            num_open_regression_stable_issues: stable_regress,
        },
        buildbots: BuildbotSummary {
            per_builder_times_mins: try!(buildbot_build_times(since, until)),
            per_builder_failures: try!(buildbot_failures_by_day(since, until)),
        },
    })
}

pub fn prs_opened_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
                       .filter(created_at.ge(since))
                       .filter(created_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_closed_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
                       .filter(closed_at.is_not_null())
                       .filter(closed_at.ge(since))
                       .filter(closed_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_merged_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("merged_at::date as d, COUNT(*)"))
                       .filter(merged_at.is_not_null())
                       .filter(merged_at.ge(since))
                       .filter(merged_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_open_time_before_close
    (since: NaiveDateTime,
     until: NaiveDateTime)
     -> DashResult<(BTreeMap<NaiveDate, i64>, BTreeMap<NaiveDate, f64>)> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());

    let w = sql::<Text>("iso_closed_week");
    let triples = try!(pullrequest.select(sql::<(BigInt, Double, Text)>("\
        COUNT(*), \
        \
        AVG(EXTRACT(EPOCH FROM closed_at) - \
          EXTRACT(EPOCH FROM created_at)) \
          / (60 * 60 * 24), \
        \
        EXTRACT(ISOYEAR FROM closed_at)::text || '-' || \
          EXTRACT(WEEK FROM closed_at)::text || '-6' AS iso_closed_week"))
                                  .filter(closed_at.is_not_null())
                                  .filter(closed_at.ge(since))
                                  .filter(closed_at.le(until))
                                  .group_by(w)
                                  .get_results::<(i64, f64, String)>(&*conn));

    let mut num_closed_map = BTreeMap::new();
    let mut days_open_map = BTreeMap::new();

    for (num_closed, days_open, week_str) in triples {
        // this will give us the end of each week we're describing
        // unwrapping this parsing is fine since we dictate above what format is acceptable
        let d = NaiveDate::parse_from_str(&week_str, "%G-%V-%w").unwrap();
        num_closed_map.insert(d, num_closed);
        days_open_map.insert(d, days_open);
    }

    Ok((num_closed_map, days_open_map))
}

pub fn open_prs_avg_days_old() -> DashResult<f64> {
    use domain::schema::pullrequest::dsl::*;
    let conn = try!(DB_POOL.get());
    Ok(try!(pullrequest.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
                       .filter(closed_at.is_null())
                       .first(&*conn)))
}

pub fn bors_retries_per_pr(since: NaiveDateTime,
                           until: NaiveDateTime)
                           -> DashResult<BTreeMap<String, i64>> {

    use domain::schema::issuecomment::dsl::*;
    let conn = try!(DB_POOL.get());

    Ok(try!(issuecomment.select(sql::<(Integer, BigInt)>("fk_issue, COUNT(*)"))
                        .filter(body.like("%@bors%retry%"))
                        .filter(created_at.ge(since))
                        .filter(created_at.le(until))
                        .group_by(fk_issue)
                        .load(&*conn))
           .into_iter()
           .map(|(k, v): (i32, i64)| (k.to_string(), v))
           .collect())
}

pub fn issues_opened_per_day(since: NaiveDateTime,
                             until: NaiveDateTime)
                             -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(issue.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
                 .filter(created_at.ge(since))
                 .filter(created_at.le(until))
                 .group_by(d)
                 .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn issues_closed_per_day(since: NaiveDateTime,
                             until: NaiveDateTime)
                             -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(issue.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
                 .filter(closed_at.is_not_null())
                 .filter(closed_at.ge(since))
                 .filter(closed_at.le(until))
                 .group_by(d)
                 .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn issues_open_time_before_close
    (since: NaiveDateTime,
     until: NaiveDateTime)
     -> DashResult<(BTreeMap<NaiveDate, i64>, BTreeMap<NaiveDate, f64>)> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());

    let w = sql::<Text>("iso_closed_week");
    let triples = try!(issue.select(sql::<(BigInt, Double, Text)>("\
        COUNT(*), \
        \
        AVG(EXTRACT(EPOCH FROM closed_at) - \
          EXTRACT(EPOCH FROM created_at)) \
          / (60 * 60 * 24), \
        \
        EXTRACT(ISOYEAR FROM closed_at)::text || '-' || \
          EXTRACT(WEEK FROM closed_at)::text || '-6' AS iso_closed_week"))
                            .filter(closed_at.is_not_null())
                            .filter(closed_at.ge(since))
                            .filter(closed_at.le(until))
                            .group_by(w)
                            .get_results::<(i64, f64, String)>(&*conn));

    let mut num_closed_map = BTreeMap::new();
    let mut days_open_map = BTreeMap::new();

    for (num_closed, days_open, week_str) in triples {
        // this will give us the end of each week we're describing
        // unwrapping this parsing is fine since we dictate above what format is acceptable
        let d = NaiveDate::parse_from_str(&week_str, "%G-%V-%w").unwrap();
        num_closed_map.insert(d, num_closed);
        days_open_map.insert(d, days_open);
    }

    Ok((num_closed_map, days_open_map))
}

pub fn open_issues_avg_days_old() -> DashResult<f64> {
    use domain::schema::issue::dsl::*;
    let conn = try!(DB_POOL.get());
    Ok(try!(issue.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
                 .filter(closed_at.is_null())
                 .first(&*conn)))
}

pub fn open_issues_with_label(label: &str) -> DashResult<i64> {
    use domain::schema::issue::dsl::*;
    let conn = try!(DB_POOL.get());

    Ok(try!(issue.select(count_star())
                 .filter(closed_at.is_not_null())
                 .filter(AsExpression::<Text>::as_expression(label).eq(any(labels)))
                 .first(&*conn)))
}

pub fn buildbot_build_times(since: NaiveDateTime,
                            until: NaiveDateTime)
                            -> DashResult<BTreeMap<String, BTreeMap<NaiveDate, f64>>> {
    use domain::schema::build::dsl::*;

    let conn = try!(DB_POOL.get());

    let name_date = sql::<(Text, Date)>("builder_name, date(start_time)");

    let triples = try!(build.select((&name_date,
                                     sql::<Double>("(AVG(duration_secs) / 60)::float")))
                            .filter(successful)
                            .filter(start_time.is_not_null())
                            .filter(start_time.ge(since))
                            .filter(start_time.le(until))
                            .filter(builder_name.like("auto-%"))
                            .group_by(&name_date)
                            .load::<((String, NaiveDate), f64)>(&*conn));

    let mut results = BTreeMap::new();
    for ((builder, date), build_minutes) in triples {
        results.entry(builder).or_insert(BTreeMap::new()).insert(date, build_minutes);
    }

    Ok(results)
}

pub fn buildbot_failures_by_day(since: NaiveDateTime,
                                until: NaiveDateTime)
                                -> DashResult<BTreeMap<String, BTreeMap<NaiveDate, i64>>> {
    use domain::schema::build::dsl::*;

    let conn = try!(DB_POOL.get());

    let name_date = sql::<(Text, Date)>("builder_name, date(start_time)");

    let triples = try!(build.select((&name_date, sql::<BigInt>("COUNT(*)")))
                            .filter(successful.ne(true))
                            .filter(start_time.is_not_null())
                            .filter(start_time.ge(since))
                            .filter(start_time.le(until))
                            .filter(builder_name.like("auto-%"))
                            .group_by(&name_date)
                            .load::<((String, NaiveDate), i64)>(&*conn));

    let mut results = BTreeMap::new();
    for ((builder, date), build_minutes) in triples {
        results.entry(builder).or_insert(BTreeMap::new()).insert(date, build_minutes);
    }

    Ok(results)
}
