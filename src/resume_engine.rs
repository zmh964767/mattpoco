use crate::models::{BaseResume, Education, JobSeeker, JobSeekerInfo, ProjectExperience, ResumeError, WorkExperience};
use rusqlite::{params, Connection};

pub struct ResumeEngine {
    conn: Connection,
}

impl ResumeEngine {
    pub fn new(db_path: &str) -> Result<Self, ResumeError> {
        let conn = Connection::open(db_path)?;
        let engine = Self { conn };
        engine.init_schema()?;
        Ok(engine)
    }

    pub fn new_in_memory() -> Result<Self, ResumeError> {
        let conn = Connection::open_in_memory()?;
        let engine = Self { conn };
        engine.init_schema()?;
        Ok(engine)
    }

    fn init_schema(&self) -> Result<(), ResumeError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS job_seekers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                phone TEXT NOT NULL DEFAULT '',
                email TEXT NOT NULL DEFAULT '',
                job_intention TEXT NOT NULL DEFAULT '[]',
                self_evaluation TEXT NOT NULL DEFAULT '',
                custom_fields TEXT NOT NULL DEFAULT '{}',
                photo_path TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS educations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_seeker_id INTEGER NOT NULL REFERENCES job_seekers(id),
                school TEXT NOT NULL,
                major TEXT NOT NULL DEFAULT '',
                degree TEXT NOT NULL DEFAULT '',
                start_date TEXT NOT NULL DEFAULT '',
                end_date TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS work_experiences (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_seeker_id INTEGER NOT NULL REFERENCES job_seekers(id),
                company TEXT NOT NULL,
                position TEXT NOT NULL DEFAULT '',
                start_date TEXT NOT NULL DEFAULT '',
                end_date TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS project_experiences (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                job_seeker_id INTEGER NOT NULL REFERENCES job_seekers(id),
                project_name TEXT NOT NULL,
                tech_stack TEXT NOT NULL DEFAULT '',
                role TEXT NOT NULL DEFAULT '',
                achievements TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS base_resumes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                job_seeker_id INTEGER NOT NULL REFERENCES job_seekers(id),
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );",
        )?;
        Ok(())
    }

    fn row_to_job_seeker(row: &rusqlite::Row) -> Result<JobSeeker, rusqlite::Error> {
        let job_intention_str: String = row.get(4)?;
        let custom_fields_str: String = row.get(6)?;
        Ok(JobSeeker {
            id: row.get(0)?,
            name: row.get(1)?,
            phone: row.get(2)?,
            email: row.get(3)?,
            job_intention: serde_json::from_str(&job_intention_str).unwrap_or_default(),
            self_evaluation: row.get(5)?,
            custom_fields: serde_json::from_str(&custom_fields_str).unwrap_or_default(),
            photo_path: row.get(7)?,
        })
    }

    pub fn create_job_seeker(&self, info: JobSeekerInfo) -> Result<JobSeeker, ResumeError> {
        self.conn.execute(
            "INSERT INTO job_seekers (name, phone, email, job_intention, self_evaluation, custom_fields, photo_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![info.name, info.phone, info.email, info.job_intention_json()?, info.self_evaluation, info.custom_fields_json()?, info.photo_path],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(JobSeeker {
            id,
            name: info.name,
            phone: info.phone,
            email: info.email,
            job_intention: info.job_intention,
            self_evaluation: info.self_evaluation,
            custom_fields: info.custom_fields,
            photo_path: info.photo_path,
        })
    }

    pub fn get_job_seeker(&self, id: i64) -> Result<JobSeeker, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, phone, email, job_intention, self_evaluation, custom_fields, photo_path FROM job_seekers WHERE id = ?1",
        )?;
        let seeker = stmt.query_row(params![id], Self::row_to_job_seeker).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => ResumeError::NotFound(id),
            other => ResumeError::Database(other),
        })?;
        Ok(seeker)
    }

    pub fn update_job_seeker(&self, id: i64, info: JobSeekerInfo) -> Result<JobSeeker, ResumeError> {
        let rows_affected = self.conn.execute(
            "UPDATE job_seekers SET name = ?1, phone = ?2, email = ?3, job_intention = ?4, self_evaluation = ?5, custom_fields = ?6, photo_path = ?7, updated_at = CURRENT_TIMESTAMP WHERE id = ?8",
            params![info.name, info.phone, info.email, info.job_intention_json()?, info.self_evaluation, info.custom_fields_json()?, info.photo_path, id],
        )?;
        if rows_affected == 0 {
            return Err(ResumeError::NotFound(id));
        }
        self.get_job_seeker(id)
    }

    pub fn add_education(&self, seeker_id: i64, school: &str, major: &str, degree: &str, start_date: &str, end_date: &str) -> Result<Education, ResumeError> {
        self.conn.execute(
            "INSERT INTO educations (job_seeker_id, school, major, degree, start_date, end_date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![seeker_id, school, major, degree, start_date, end_date],
        )?;
        Ok(Education {
            id: self.conn.last_insert_rowid(),
            job_seeker_id: seeker_id,
            school: school.to_string(),
            major: major.to_string(),
            degree: degree.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
        })
    }

    pub fn list_educations(&self, seeker_id: i64) -> Result<Vec<Education>, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_seeker_id, school, major, degree, start_date, end_date FROM educations WHERE job_seeker_id = ?1",
        )?;
        let results = stmt.query_map(params![seeker_id], |row| {
            Ok(Education {
                id: row.get(0)?,
                job_seeker_id: row.get(1)?,
                school: row.get(2)?,
                major: row.get(3)?,
                degree: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
            })
        })?;
        results.collect::<Result<Vec<_>, _>>().map_err(ResumeError::Database)
    }

    pub fn add_work_experience(&self, seeker_id: i64, company: &str, position: &str, start_date: &str, end_date: &str, description: &str) -> Result<WorkExperience, ResumeError> {
        self.conn.execute(
            "INSERT INTO work_experiences (job_seeker_id, company, position, start_date, end_date, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![seeker_id, company, position, start_date, end_date, description],
        )?;
        Ok(WorkExperience {
            id: self.conn.last_insert_rowid(),
            job_seeker_id: seeker_id,
            company: company.to_string(),
            position: position.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            description: description.to_string(),
        })
    }

    pub fn list_work_experiences(&self, seeker_id: i64) -> Result<Vec<WorkExperience>, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_seeker_id, company, position, start_date, end_date, description FROM work_experiences WHERE job_seeker_id = ?1",
        )?;
        let results = stmt.query_map(params![seeker_id], |row| {
            Ok(WorkExperience {
                id: row.get(0)?,
                job_seeker_id: row.get(1)?,
                company: row.get(2)?,
                position: row.get(3)?,
                start_date: row.get(4)?,
                end_date: row.get(5)?,
                description: row.get(6)?,
            })
        })?;
        results.collect::<Result<Vec<_>, _>>().map_err(ResumeError::Database)
    }

    pub fn add_project_experience(&self, seeker_id: i64, project_name: &str, tech_stack: &str, role: &str, achievements: &str) -> Result<ProjectExperience, ResumeError> {
        self.conn.execute(
            "INSERT INTO project_experiences (job_seeker_id, project_name, tech_stack, role, achievements) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![seeker_id, project_name, tech_stack, role, achievements],
        )?;
        Ok(ProjectExperience {
            id: self.conn.last_insert_rowid(),
            job_seeker_id: seeker_id,
            project_name: project_name.to_string(),
            tech_stack: tech_stack.to_string(),
            role: role.to_string(),
            achievements: achievements.to_string(),
        })
    }

    pub fn list_project_experiences(&self, seeker_id: i64) -> Result<Vec<ProjectExperience>, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_seeker_id, project_name, tech_stack, role, achievements FROM project_experiences WHERE job_seeker_id = ?1",
        )?;
        let results = stmt.query_map(params![seeker_id], |row| {
            Ok(ProjectExperience {
                id: row.get(0)?,
                job_seeker_id: row.get(1)?,
                project_name: row.get(2)?,
                tech_stack: row.get(3)?,
                role: row.get(4)?,
                achievements: row.get(5)?,
            })
        })?;
        results.collect::<Result<Vec<_>, _>>().map_err(ResumeError::Database)
    }

    pub fn create_base_resume(&self, seeker_id: i64, name: &str) -> Result<BaseResume, ResumeError> {
        self.conn.execute(
            "INSERT INTO base_resumes (name, job_seeker_id) VALUES (?1, ?2)",
            params![name, seeker_id],
        )?;
        Ok(BaseResume {
            id: self.conn.last_insert_rowid(),
            name: name.to_string(),
            job_seeker_id: seeker_id,
        })
    }

    pub fn list_base_resumes(&self, seeker_id: i64) -> Result<Vec<BaseResume>, ResumeError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, job_seeker_id FROM base_resumes WHERE job_seeker_id = ?1",
        )?;
        let results = stmt.query_map(params![seeker_id], |row| {
            Ok(BaseResume {
                id: row.get(0)?,
                name: row.get(1)?,
                job_seeker_id: row.get(2)?,
            })
        })?;
        results.collect::<Result<Vec<_>, _>>().map_err(ResumeError::Database)
    }

    pub fn delete_base_resume(&self, id: i64) -> Result<(), ResumeError> {
        let rows_affected = self.conn.execute("DELETE FROM base_resumes WHERE id = ?1", params![id])?;
        if rows_affected == 0 {
            return Err(ResumeError::NotFound(id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::JobSeekerInfo;

    fn create_engine() -> ResumeEngine {
        ResumeEngine::new_in_memory().unwrap()
    }

    fn sample_seeker_info(name: &str) -> JobSeekerInfo {
        JobSeekerInfo {
            name: name.to_string(),
            phone: "13800138000".to_string(),
            email: format!("{}@example.com", name),
            job_intention: vec!["前端开发".to_string()],
            self_evaluation: "3年经验".to_string(),
            custom_fields: serde_json::json!({}),
            photo_path: None,
        }
    }

    #[test]
    fn create_job_seeker_stores_and_retrieves() {
        let engine = create_engine();
        let info = JobSeekerInfo {
            name: "张三".to_string(),
            phone: "13800138000".to_string(),
            email: "zhangsan@example.com".to_string(),
            job_intention: vec!["前端开发".to_string(), "全栈开发".to_string()],
            self_evaluation: "3年前端开发经验".to_string(),
            custom_fields: serde_json::json!({"政治面貌": "党员"}),
            photo_path: Some("/photos/zhangsan.jpg".to_string()),
        };

        let created = engine.create_job_seeker(info).unwrap();
        assert!(created.id > 0);

        let retrieved = engine.get_job_seeker(created.id).unwrap();
        assert_eq!(retrieved.name, "张三");
        assert_eq!(retrieved.phone, "13800138000");
        assert_eq!(retrieved.email, "zhangsan@example.com");
        assert_eq!(retrieved.job_intention, vec!["前端开发", "全栈开发"]);
        assert_eq!(retrieved.self_evaluation, "3年前端开发经验");
        assert_eq!(retrieved.custom_fields["政治面貌"], "党员");
        assert_eq!(retrieved.photo_path, Some("/photos/zhangsan.jpg".to_string()));
    }

    #[test]
    fn get_job_seeker_not_found() {
        let engine = create_engine();
        let result = engine.get_job_seeker(999);
        assert!(matches!(result, Err(ResumeError::NotFound(999))));
    }

    #[test]
    fn update_job_seeker_modifies_fields() {
        let engine = create_engine();
        let created = engine.create_job_seeker(sample_seeker_info("李四")).unwrap();

        let updated_info = JobSeekerInfo {
            name: "李四更新".to_string(),
            phone: "13900139999".to_string(),
            email: "lisi_new@example.com".to_string(),
            job_intention: vec!["后端开发".to_string(), "架构师".to_string()],
            self_evaluation: "5年后端经验".to_string(),
            custom_fields: serde_json::json!({"民族": "汉族"}),
            photo_path: Some("/photos/lisi.jpg".to_string()),
        };
        let updated = engine.update_job_seeker(created.id, updated_info).unwrap();
        assert_eq!(updated.name, "李四更新");
        assert_eq!(updated.phone, "13900139999");
        assert_eq!(updated.email, "lisi_new@example.com");
        assert_eq!(updated.job_intention.len(), 2);
        assert_eq!(updated.self_evaluation, "5年后端经验");
    }

    #[test]
    fn update_job_seeker_not_found() {
        let engine = create_engine();
        let result = engine.update_job_seeker(999, sample_seeker_info("不存在"));
        assert!(matches!(result, Err(ResumeError::NotFound(999))));
    }

    #[test]
    fn add_and_list_educations() {
        let engine = create_engine();
        let seeker = engine.create_job_seeker(sample_seeker_info("王五")).unwrap();

        engine.add_education(seeker.id, "清华大学", "计算机科学", "本科", "2018-09", "2022-06").unwrap();
        engine.add_education(seeker.id, "北京大学", "人工智能", "硕士", "2022-09", "2025-06").unwrap();

        let educations = engine.list_educations(seeker.id).unwrap();
        assert_eq!(educations.len(), 2);
        assert_eq!(educations[0].school, "清华大学");
        assert_eq!(educations[1].major, "人工智能");
    }

    #[test]
    fn add_and_list_work_experiences() {
        let engine = create_engine();
        let seeker = engine.create_job_seeker(sample_seeker_info("赵六")).unwrap();

        engine.add_work_experience(seeker.id, "字节跳动", "前端工程师", "2022-07", "2024-03", "抖音Web端").unwrap();
        engine.add_work_experience(seeker.id, "阿里巴巴", "高级前端", "2024-04", "", "淘宝首页").unwrap();

        let works = engine.list_work_experiences(seeker.id).unwrap();
        assert_eq!(works.len(), 2);
        assert_eq!(works[0].company, "字节跳动");
        assert_eq!(works[1].position, "高级前端");
    }

    #[test]
    fn add_and_list_project_experiences() {
        let engine = create_engine();
        let seeker = engine.create_job_seeker(sample_seeker_info("孙七")).unwrap();

        engine.add_project_experience(seeker.id, "电商系统", "React+Node.js", "全栈开发", "日活提升30%").unwrap();
        engine.add_project_experience(seeker.id, "推荐引擎", "Rust+Python", "核心开发", "CTR提升15%").unwrap();

        let projects = engine.list_project_experiences(seeker.id).unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].project_name, "电商系统");
        assert_eq!(projects[1].tech_stack, "Rust+Python");
    }

    #[test]
    fn create_and_list_base_resumes() {
        let engine = create_engine();
        let seeker = engine.create_job_seeker(sample_seeker_info("周八")).unwrap();

        engine.create_base_resume(seeker.id, "互联网版").unwrap();
        engine.create_base_resume(seeker.id, "国企版").unwrap();

        let resumes = engine.list_base_resumes(seeker.id).unwrap();
        assert_eq!(resumes.len(), 2);
        assert_eq!(resumes[0].name, "互联网版");
        assert_eq!(resumes[1].name, "国企版");
    }

    #[test]
    fn delete_base_resume_removes_from_list() {
        let engine = create_engine();
        let seeker = engine.create_job_seeker(sample_seeker_info("吴九")).unwrap();

        let r1 = engine.create_base_resume(seeker.id, "简历A").unwrap();
        engine.create_base_resume(seeker.id, "简历B").unwrap();

        engine.delete_base_resume(r1.id).unwrap();
        let resumes = engine.list_base_resumes(seeker.id).unwrap();
        assert_eq!(resumes.len(), 1);
        assert_eq!(resumes[0].name, "简历B");
    }

    #[test]
    fn delete_base_resume_not_found() {
        let engine = create_engine();
        let result = engine.delete_base_resume(999);
        assert!(matches!(result, Err(ResumeError::NotFound(999))));
    }
}
