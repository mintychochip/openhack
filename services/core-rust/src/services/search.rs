use crate::errors::CoreError;
use crate::models::search::{ProjectResult, SearchQuery, SearchResult, TeamResult, UserResult};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct SearchService;

impl SearchService {
    pub async fn search(
        pool: &PgPool,
        query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, CoreError> {
        let limit = query.limit.unwrap_or(20);
        let offset = query.offset.unwrap_or(0);

        let types_filter = if let Some(ref types) = query.types {
            types.split(',').collect::<Vec<_>>()
        } else {
            vec!["project", "team", "user"]
        };

        let mut results = Vec::new();



        if types_filter.contains(&"project") {
            let rows = sqlx::query(
                r#"SELECT 
                    p.id,
                    p.name as title,
                    p.description,
                    t.name as team_name,
                    p.tags,
                    p.featured,
                    p.favorite_count,
                    ts_rank(p.search_vector, plainto_tsquery('english', $1)) as rank
                FROM core.projects p
                LEFT JOIN core.teams t ON t.id = p.team_id
                WHERE p.search_vector @@ plainto_tsquery('english', $1)
                ORDER BY rank DESC
                LIMIT $2 OFFSET $3"#
            )
            .bind(&query.q)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for row in rows {
                let proj = ProjectResult {
                    id: row.get::<Uuid, _>(0).to_string(),
                    title: row.get::<String, _>(1),
                    description: row.get::<Option<String>, _>(2),
                    team_name: row.get::<Option<String>, _>(3),
                    tags: row.get::<Option<Vec<String>>, _>(4),
                    featured: row.get::<bool, _>(5),
                    favorite_count: row.get::<i32, _>(6),
                    rank: row.get::<f64, _>(7),
                };
                results.push(SearchResult::Project(proj));
            }
        }

        if types_filter.contains(&"team") {
            let rows = sqlx::query(
                r#"SELECT 
                    t.id,
                    t.name,
                    t.description,
                    (SELECT COUNT(*) FROM core.team_members WHERE team_id = t.id) as member_count,
                    t.looking_for_members,
                    COALESCE(t.seeking_skills, '{}') as seeking_skills,
                    ts_rank(t.search_vector, plainto_tsquery('english', $1)) as rank
                FROM core.teams t
                WHERE t.search_vector @@ plainto_tsquery('english', $1)
                ORDER BY rank DESC
                LIMIT $2 OFFSET $3"#
            )
            .bind(&query.q)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for row in rows {
                let team = TeamResult {
                    id: row.get::<Uuid, _>(0).to_string(),
                    name: row.get::<String, _>(1),
                    description: row.get::<Option<String>, _>(2),
                    member_count: row.get::<i32, _>(3),
                    looking_for_members: row.get::<bool, _>(4),
                    seeking_skills: row.get::<Vec<String>, _>(5),
                    rank: row.get::<f64, _>(6),
                };
                results.push(SearchResult::Team(team));
            }
        }

        if types_filter.contains(&"user") {
            let rows = sqlx::query(
                r#"SELECT 
                    u.id,
                    u.name,
                    u.email,
                    COALESCE(u.skills, '{}') as skills,
                    u.looking_for_team,
                    ts_rank(u.search_vector, plainto_tsquery('english', $1)) as rank
                FROM auth.users u
                WHERE u.search_vector @@ plainto_tsquery('english', $1)
                ORDER BY rank DESC
                LIMIT $2 OFFSET $3"#
            )
            .bind(&query.q)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

            for row in rows {
                let user = UserResult {
                    id: row.get::<Uuid, _>(0).to_string(),
                    name: row.get::<String, _>(1),
                    email: row.get::<Option<String>, _>(2),
                    skills: row.get::<Vec<String>, _>(3),
                    looking_for_team: row.get::<bool, _>(4),
                    rank: row.get::<f64, _>(5),
                };
                results.push(SearchResult::User(user));
            }
        }

        Ok(results)
    }
}
