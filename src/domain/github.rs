// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, FilterDsl, LoadDsl, Queryable, SaveChangesDsl, Table};

use super::schema::*;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(githubuser)]
#[changeset_for(githubuser)]
pub struct GitHubUser {
    pub id: i32,
    pub login: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(milestone)]
#[changeset_for(milestone, treat_none_as_null="true")]
pub struct Milestone {
    pub id: i32,
    pub number: i32,
    pub open: bool,
    pub title: String,
    pub description: Option<String>,
    pub fk_creator: i32,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub due_on: Option<NaiveDateTime>,
    pub repository: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[insertable_into(issue)]
pub struct IssuePartial {
    pub number: i32,
    pub fk_milestone: Option<i32>,
    pub fk_user: i32,
    pub fk_assignee: Option<i32>,
    pub open: bool,
    pub is_pull_request: bool,
    pub title: String,
    pub body: String,
    pub locked: bool,
    pub closed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub labels: Vec<String>,
    pub repository: String,
}

impl IssuePartial {
    pub fn complete(self, id: i32) -> Issue {
        Issue {
            id: id,
            number: self.number,
            fk_milestone: self.fk_milestone,
            fk_user: self.fk_user,
            fk_assignee: self.fk_assignee,
            open: self.open,
            is_pull_request: self.is_pull_request,
            title: self.title,
            body: self.body,
            locked: self.locked,
            closed_at: self.closed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
            labels: self.labels,
            repository: self.repository,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[changeset_for(issue, treat_none_as_null="true")]
pub struct Issue {
    pub id: i32,
    pub number: i32,
    pub fk_milestone: Option<i32>,
    pub fk_user: i32,
    pub fk_assignee: Option<i32>,
    pub open: bool,
    pub is_pull_request: bool,
    pub title: String,
    pub body: String,
    pub locked: bool,
    pub closed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub labels: Vec<String>,
    pub repository: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(issuecomment)]
#[changeset_for(issuecomment, treat_none_as_null="true")]
pub struct IssueComment {
    pub id: i32,
    pub fk_issue: i32,
    pub fk_user: i32,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub repository: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(pullrequest)]
#[changeset_for(pullrequest, treat_none_as_null="true")]
pub struct PullRequest {
    pub number: i32,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub fk_assignee: Option<i32>,
    pub fk_milestone: Option<i32>,
    pub locked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub merged_at: Option<NaiveDateTime>,
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
    pub repository: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(teams)]
pub struct Team {
    pub id: i32,
    pub name: String,
    pub ping: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(memberships)]
pub struct Membership {
    pub id: i32,
    pub fk_member: i32,
    pub fk_team: i32,
}
