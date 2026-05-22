use crate::errors::CoreError;
use crate::models::project::{FavoriteRequest, FavoriteResponse, FeatureRequest, ShowcaseQuery};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

pub struct ShowcaseService;

impl ShowcaseService {
    pub async fn get_showcase(
        pool: &PgPool,
        query: &ShowcaseQuery,
        user_id: Option<Uuid>,
    ) -> Result<crate::models::project::ProjectListResponse, CoreError> {
        let limit = query.page_size.unwrap_or(20);
        let offset = (query.page.unwrap_or(1) - 1) * limit;

        let sort = query.sort.as_deref().unwrap_or("featured");
        
        let order_clause: &str = match sort {
            "popular" => "ORDER BY favorite_count DESC, created_at DESC",
            "recent" => "ORDER BY created_at DESC",
            "featured" => "ORDER BY featured DESC, showcase_order ASC, created_at DESC",
            _ => "ORDER BY featured DESC, showcase_order ASC, created_at DESC",
        };

        let category_filter = if let Some(ref cat) = query.category {
            "AND category = $1"
        } else {
            ""
        };

        let count_query = format!(
            "SELECT COUNT(*) FROM core.projects WHERE featured = TRUE {}",
            if query.category.is_some() { category_filter } else { "" }
        );

        let total: i64 = if let Some(ref cat) = query.category {
            sqlx::query_scalar(&count_query)
                .bind(cat)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        } else {
            sqlx::query_scalar(&count_query)
                .fetch_one(pool)
                .await
                .unwrap_or(0)
        };

        let data_query = format!(
            "SELECT * FROM core.projects WHERE featured = TRUE {} {} LIMIT $1 OFFSET $2",
            category_filter, order_clause
        );

        let mut query_builder = sqlx::query_as::<_, crate::models::project::Project>(&data_query);

        if let Some(ref cat) = query.category {
            query_builder = query_builder.bind(cat);
        }

        let projects = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let mut responses: Vec<crate::models::project::ProjectResponse> = 
            projects.into_iter().map(Into::into).collect();

        if let Some(uid) = user_id {
            let project_ids: Vec<Uuid> = responses.iter().map(|p| p.id).collect();
            if !project_ids.is_empty() {
                let favs: Vec<(Uuid,)> = sqlx::query_as(
                    "SELECT project_id FROM core.project_favorites WHERE user_id = $1 AND project_id = ANY($2)"
                )
                .bind(uid)
                .bind(&project_ids)
                .fetch_all(pool)
                .await
                .unwrap_or_default();

                let fav_set: std::collections::HashSet<Uuid> = favs.into_iter().map(|(pid,)| pid).collect();
                for proj in &mut responses {
                    proj.is_favorite = Some(fav_set.contains(&proj.id));
                }
            }
        }

        Ok(crate::models::project::ProjectListResponse {
            projects: responses,
            total,
        })
    }

    pub async fn set_featured(
        pool: &PgPool,
        project_id: Uuid,
        req: &FeatureRequest,
    ) -> Result<crate::models::project::ProjectResponse, CoreError> {
        let row = sqlx::query(
            "UPDATE core.projects SET featured = $1, showcase_order = COALESCE($2, showcase_order) WHERE id = $3 RETURNING *"
        )
        .bind(req.featured)
        .bind(req.showcase_order)
        .bind(project_id)
        .fetch_one(pool)
        .await?;

        let project: crate::models::project::Project = sqlx::FromRow::from_row(&row)?;
        Ok(project.into())
    }

    pub async fn toggle_favorite(
        pool: &PgPool,
        project_id: Uuid,
        user_id: Uuid,
        req: &FavoriteRequest,
    ) -> Result<FavoriteResponse, CoreError> {
        if req.favorite {
            sqlx::query(
                "INSERT INTO core.project_favorites (user_id, project_id) VALUES ($1, $2) 
                 ON CONFLICT (user_id, project_id) DO NOTHING"
            )
            .bind(user_id)
            .bind(project_id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                "DELETE FROM core.project_favorites WHERE user_id = $1 AND project_id = $2"
            )
            .bind(user_id)
            .bind(project_id)
            .execute(pool)
            .await?;
        }

        let (favorite_count, is_favorite) = sqlx::query_as::<_, (i32, bool)>(
            "SELECT 
                (SELECT COUNT(*) FROM core.project_favorites WHERE project_id = $1),
                EXISTS(SELECT 1 FROM core.project_favorites WHERE project_id = $1 AND user_id = $2)"
        )
        .bind(project_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(FavoriteResponse {
            project_id,
            favorite_count,
            is_favorite,
        })
    }

    pub async fn get_favorite_info(
        pool: &PgPool,
        project_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<FavoriteResponse, CoreError> {
        let (favorite_count, is_favorite) = if let Some(uid) = user_id {
            sqlx::query_as::<_, (i32, bool)>(
                "SELECT 
                    (SELECT COUNT(*) FROM core.project_favorites WHERE project_id = $1),
                    EXISTS(SELECT 1 FROM core.project_favorites WHERE project_id = $1 AND user_id = $2)"
            )
            .bind(project_id)
            .bind(uid)
            .fetch_one(pool)
            .await?
        } else {
            let count: i32 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM core.project_favorites WHERE project_id = $1"
            )
            .bind(project_id)
            .fetch_one(pool)
            .await
            .unwrap_or(0);
            (count, false)
        };

        Ok(FavoriteResponse {
            project_id,
            favorite_count,
            is_favorite,
        })
    }
}
