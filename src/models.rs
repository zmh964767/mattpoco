use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSeeker {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub job_intention: Vec<String>,
    pub self_evaluation: String,
    pub custom_fields: serde_json::Value,
    pub photo_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSeekerInfo {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub job_intention: Vec<String>,
    pub self_evaluation: String,
    pub custom_fields: serde_json::Value,
    pub photo_path: Option<String>,
}

impl JobSeekerInfo {
    pub fn job_intention_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.job_intention)
    }

    pub fn custom_fields_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.custom_fields)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Education {
    pub id: i64,
    pub job_seeker_id: i64,
    pub school: String,
    pub major: String,
    pub degree: String,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkExperience {
    pub id: i64,
    pub job_seeker_id: i64,
    pub company: String,
    pub position: String,
    pub start_date: String,
    pub end_date: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExperience {
    pub id: i64,
    pub job_seeker_id: i64,
    pub project_name: String,
    pub tech_stack: String,
    pub role: String,
    pub achievements: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub platform: String,
    pub job_title: String,
    pub company_name: String,
    pub jd_text: String,
    pub salary_range: String,
    pub location: String,
    pub skills_required: Vec<String>,
    pub experience_req: String,
    pub education_req: String,
    pub match_score: Option<f64>,
    pub score_details: Option<serde_json::Value>,
    pub crawl_time: String,
    pub is_expired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub platform: String,
    pub job_title: String,
    pub company_name: String,
    pub jd_text: String,
    pub salary_range: String,
    pub location: String,
    pub skills_required: Vec<String>,
    pub experience_req: String,
    pub education_req: String,
}

impl JobInfo {
    pub fn skills_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.skills_required)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseResume {
    pub id: i64,
    pub name: String,
    pub job_seeker_id: i64,
}

#[derive(Debug)]
pub enum ResumeError {
    NotFound(i64),
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for ResumeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResumeError::NotFound(id) => write!(f, "资源未找到: id={}", id),
            ResumeError::Database(e) => write!(f, "数据库错误: {}", e),
            ResumeError::Serialization(e) => write!(f, "序列化错误: {}", e),
        }
    }
}

impl std::error::Error for ResumeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ResumeError::Database(e) => Some(e),
            ResumeError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for ResumeError {
    fn from(e: rusqlite::Error) -> Self {
        ResumeError::Database(e)
    }
}

impl From<serde_json::Error> for ResumeError {
    fn from(e: serde_json::Error) -> Self {
        ResumeError::Serialization(e)
    }
}
