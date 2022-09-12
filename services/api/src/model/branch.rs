use std::str::FromStr;

use bencher_json::{JsonBranch, JsonNewBranch};
use diesel::{ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl, SqliteConnection};
use dropshot::HttpError;
use uuid::Uuid;

use super::project::QueryProject;
use crate::{
    schema,
    schema::branch as branch_table,
    util::{map_http_error, slug::validate_slug},
};

#[derive(Queryable)]
pub struct QueryBranch {
    pub id: i32,
    pub uuid: String,
    pub project_id: i32,
    pub name: String,
    pub slug: String,
}

impl QueryBranch {
    pub fn get_id(conn: &mut SqliteConnection, uuid: impl ToString) -> Result<i32, HttpError> {
        schema::branch::table
            .filter(schema::branch::uuid.eq(uuid.to_string()))
            .select(schema::branch::id)
            .first(conn)
            .map_err(map_http_error!("Failed to get branch."))
    }

    pub fn get_uuid(conn: &mut SqliteConnection, id: i32) -> Result<Uuid, HttpError> {
        let uuid: String = schema::branch::table
            .filter(schema::branch::id.eq(id))
            .select(schema::branch::uuid)
            .first(conn)
            .map_err(map_http_error!("Failed to get branch."))?;
        Uuid::from_str(&uuid).map_err(map_http_error!("Failed to get branch."))
    }

    pub fn into_json(self, conn: &mut SqliteConnection) -> Result<JsonBranch, HttpError> {
        let Self {
            id: _,
            uuid,
            project_id,
            name,
            slug,
        } = self;
        Ok(JsonBranch {
            uuid: Uuid::from_str(&uuid).map_err(map_http_error!("Failed to get branch."))?,
            project: QueryProject::get_uuid(conn, project_id)?,
            name,
            slug,
        })
    }
}

#[derive(Insertable)]
#[diesel(table_name = branch_table)]
pub struct InsertBranch {
    pub uuid: String,
    pub project_id: i32,
    pub name: String,
    pub slug: String,
}

impl InsertBranch {
    pub fn from_json(
        conn: &mut SqliteConnection,
        branch: JsonNewBranch,
    ) -> Result<Self, HttpError> {
        let JsonNewBranch {
            project,
            name,
            slug,
        } = branch;
        let slug = validate_slug(
            conn,
            &name,
            slug,
            Box::new(|conn, slug| {
                schema::branch::table
                    .filter(schema::branch::slug.eq(slug))
                    .first::<QueryBranch>(conn)
                    .is_ok()
            }),
        );
        Ok(Self {
            uuid: Uuid::new_v4().to_string(),
            project_id: QueryProject::from_resource_id(conn, &project)?.id,
            name,
            slug,
        })
    }
}
