use crate::models::{Job, JobInfo, ResumeError};
use rusqlite::{params, Connection};

pub struct JobCrawler {
    conn: Connection,
}

impl JobCrawler {
    pub fn new(db_path: &str) -> Result<Self, ResumeError> {
        let conn = Connection::open(db_path)?;
        let crawler = Self { conn };
        crawler.init_schema()?;
        Ok(crawler)
    }

    pub fn new_in_memory() -> Result<Self, ResumeError> {
        let conn = Connection::open_in_memory()?;
        let crawler = Self { conn };
        crawler.init_schema()?;
        Ok(crawler)
    }

    fn init_schema(&self) -> Result<(), ResumeError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                platform TEXT NOT NULL,
                job_title TEXT NOT NULL,
                company_name TEXT NOT NULL,
                jd_text TEXT NOT NULL DEFAULT '',
                salary_range TEXT NOT NULL DEFAULT '',
                location TEXT NOT NULL DEFAULT '',
                skills_required TEXT NOT NULL DEFAULT '[]',
                experience_req TEXT NOT NULL DEFAULT '',
                education_req TEXT NOT NULL DEFAULT '',
                match_score REAL,
                score_details TEXT,
                crawl_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_expired INTEGER NOT NULL DEFAULT 0
            );",
        )?;
        Ok(())
    }

    fn row_to_job(row: &rusqlite::Row) -> Result<Job, rusqlite::Error> {
        let skills_str: String = row.get(7)?;
        let score_details_str: Option<String> = row.get(11)?;
        Ok(Job {
            id: row.get(0)?,
            platform: row.get(1)?,
            job_title: row.get(2)?,
            company_name: row.get(3)?,
            jd_text: row.get(4)?,
            salary_range: row.get(5)?,
            location: row.get(6)?,
            skills_required: serde_json::from_str(&skills_str).unwrap_or_default(),
            experience_req: row.get(8)?,
            education_req: row.get(9)?,
            match_score: row.get(10)?,
            score_details: score_details_str.and_then(|s| serde_json::from_str(&s).ok()),
            crawl_time: row.get(12)?,
            is_expired: row.get::<_, i32>(13)? != 0,
        })
    }

    pub fn save_job(&self, info: JobInfo) -> Result<Job, ResumeError> {
        self.conn.execute(
            "INSERT INTO jobs (platform, job_title, company_name, jd_text, salary_range, location, skills_required, experience_req, education_req) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![info.platform, info.job_title, info.company_name, info.jd_text, info.salary_range, info.location, info.skills_json()?, info.experience_req, info.education_req],
        )?;
        let id = self.conn.last_insert_rowid();
        self.get_job(id)
    }

    pub fn get_job(&self, id: i64) -> Result<Job, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, platform, job_title, company_name, jd_text, salary_range, location, skills_required, experience_req, education_req, match_score, score_details, crawl_time, is_expired FROM jobs WHERE id = ?1",
        )?;
        let job = stmt.query_row(params![id], Self::row_to_job).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ResumeError::NotFound(id),
            other => ResumeError::Database(other),
        })?;
        Ok(job)
    }

    pub fn list_jobs(&self, platform: Option<&str>) -> Result<Vec<Job>, ResumeError> {
        let sql = match platform {
            Some(_) => "SELECT id, platform, job_title, company_name, jd_text, salary_range, location, skills_required, experience_req, education_req, match_score, score_details, crawl_time, is_expired FROM jobs WHERE platform = ?1 AND is_expired = 0 ORDER BY crawl_time DESC",
            None => "SELECT id, platform, job_title, company_name, jd_text, salary_range, location, skills_required, experience_req, education_req, match_score, score_details, crawl_time, is_expired FROM jobs WHERE is_expired = 0 ORDER BY crawl_time DESC",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let results = match platform {
            Some(p) => stmt.query_map(params![p], Self::row_to_job)?,
            None => stmt.query_map(params![], Self::row_to_job)?,
        };
        results.collect::<Result<Vec<_>, _>>().map_err(ResumeError::Database)
    }

    pub fn mark_expired(&self, id: i64) -> Result<(), ResumeError> {
        let rows = self.conn.execute("UPDATE jobs SET is_expired = 1 WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(ResumeError::NotFound(id));
        }
        Ok(())
    }

    pub fn expire_old_jobs(&self, days: i64) -> Result<usize, ResumeError> {
        let rows = self.conn.execute(
            "UPDATE jobs SET is_expired = 1 WHERE is_expired = 0 AND crawl_time < datetime('now', ?1)",
            params![format!("-{} days", days)],
        )?;
        Ok(rows)
    }

    pub fn update_match_score(&self, id: i64, score: f64, details: serde_json::Value) -> Result<(), ResumeError> {
        let details_json = serde_json::to_string(&details)?;
        let rows = self.conn.execute(
            "UPDATE jobs SET match_score = ?1, score_details = ?2 WHERE id = ?3",
            params![score, details_json, id],
        )?;
        if rows == 0 {
            return Err(ResumeError::NotFound(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_crawler() -> JobCrawler {
        JobCrawler::new_in_memory().unwrap()
    }

    fn sample_job(platform: &str, title: &str) -> JobInfo {
        JobInfo {
            platform: platform.to_string(),
            job_title: title.to_string(),
            company_name: "测试公司".to_string(),
            jd_text: "负责前端开发".to_string(),
            salary_range: "15k-25k".to_string(),
            location: "北京".to_string(),
            skills_required: vec!["React".to_string(), "TypeScript".to_string()],
            experience_req: "3年".to_string(),
            education_req: "本科".to_string(),
        }
    }

    #[test]
    fn save_and_retrieve_job() {
        let crawler = create_crawler();
        let info = sample_job("boss", "前端工程师");
        let saved = crawler.save_job(info).unwrap();

        assert!(saved.id > 0);
        assert_eq!(saved.platform, "boss");
        assert_eq!(saved.job_title, "前端工程师");
        assert_eq!(saved.skills_required, vec!["React", "TypeScript"]);
        assert!(!saved.is_expired);
    }

    #[test]
    fn list_jobs_by_platform() {
        let crawler = create_crawler();
        crawler.save_job(sample_job("boss", "前端工程师")).unwrap();
        crawler.save_job(sample_job("boss", "后端工程师")).unwrap();
        crawler.save_job(sample_job("zhaopin", "全栈工程师")).unwrap();

        let boss_jobs = crawler.list_jobs(Some("boss")).unwrap();
        assert_eq!(boss_jobs.len(), 2);

        let all_jobs = crawler.list_jobs(None).unwrap();
        assert_eq!(all_jobs.len(), 3);
    }

    #[test]
    fn mark_job_expired() {
        let crawler = create_crawler();
        let job = crawler.save_job(sample_job("boss", "前端工程师")).unwrap();

        crawler.mark_expired(job.id).unwrap();
        let retrieved = crawler.get_job(job.id).unwrap();
        assert!(retrieved.is_expired);

        let active_jobs = crawler.list_jobs(None).unwrap();
        assert_eq!(active_jobs.len(), 0);
    }

    #[test]
    fn expire_old_jobs() {
        let crawler = create_crawler();
        crawler.save_job(sample_job("boss", "前端工程师")).unwrap();

        crawler.conn.execute(
            "UPDATE jobs SET crawl_time = datetime('now', '-10 days') WHERE id = 1",
            params![],
        ).unwrap();

        let expired_count = crawler.expire_old_jobs(7).unwrap();
        assert_eq!(expired_count, 1);

        let active_jobs = crawler.list_jobs(None).unwrap();
        assert_eq!(active_jobs.len(), 0);
    }

    #[test]
    fn update_match_score() {
        let crawler = create_crawler();
        let job = crawler.save_job(sample_job("boss", "前端工程师")).unwrap();

        let details = serde_json::json!({"skills": 0.9, "experience": 0.7});
        crawler.update_match_score(job.id, 0.85, details).unwrap();

        let updated = crawler.get_job(job.id).unwrap();
        assert!((updated.match_score.unwrap() - 0.85).abs() < 0.001);
        assert_eq!(updated.score_details.as_ref().unwrap()["skills"], 0.9);
    }

    #[test]
    fn get_job_not_found() {
        let crawler = create_crawler();
        let result = crawler.get_job(999);
        assert!(matches!(result, Err(ResumeError::NotFound(999))));
    }
}
